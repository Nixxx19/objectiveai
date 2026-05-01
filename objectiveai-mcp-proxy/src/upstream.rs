//! Parsing of the proxy's three custom session-init headers and fan-out
//! connect over the resulting upstream specs.

use axum::http::HeaderMap;
use futures::future::try_join_all;
use indexmap::IndexMap;
use objectiveai::mcp::{Client, Connection};

const SERVERS_HEADER: &str = "X-MCP-Servers";
const HEADERS_HEADER: &str = "X-MCP-Headers";
const AUTHORIZATION_HEADER: &str = "X-MCP-Authorization";

/// One upstream MCP server the proxy should connect to for a session.
#[derive(Debug)]
struct UpstreamSpec {
    url: String,
    authorization: Option<String>,
    extra_headers: IndexMap<String, String>,
}

/// Why parsing the three custom session-init headers failed, or why an
/// upstream connect failed.
#[derive(Debug, thiserror::Error)]
pub enum BadInit {
    #[error("{header} is not valid UTF-8")]
    NotUtf8 { header: &'static str },
    #[error("{header} is not valid JSON: {source}")]
    NotJson {
        header: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error("upstream connect failed for {url}: {source}")]
    UpstreamConnectFailed {
        url: String,
        #[source]
        source: objectiveai::mcp::Error,
    },
}

/// HTTP header name used to carry the upstream MCP session id. Stored
/// alongside `Authorization` and any custom headers in the per-upstream
/// header map encoded into the proxy session id.
pub const MCP_SESSION_ID_KEY: &str = "Mcp-Session-Id";
/// HTTP header name for the upstream's bearer-token Authorization.
pub const AUTHORIZATION_KEY: &str = "Authorization";

/// Parse the three custom session-init headers and fresh-connect to
/// every upstream URL they describe in parallel.
///
/// This is the no-prior-session path: every URL is connected from
/// scratch, no resume sid. The resume / re-encode flow lives in
/// `mcp::handle_initialize` and uses [`reconnect_from_payload`].
///
/// Headers (all optional):
/// - `X-MCP-Servers`: JSON array of upstream URLs. Empty / absent → empty
///   Vec is returned (the session still initializes, the client just gets
///   nothing from `tools/list` etc).
/// - `X-MCP-Headers`: JSON `{string: string}` of extra HTTP headers
///   forwarded on every upstream request.
/// - `X-MCP-Authorization`: JSON `{url: string}` per-URL `Authorization`
///   value. Overrides whatever `X-MCP-Headers` would have sent for that URL.
///
/// Returns each opened `Connection` paired with the canonical full
/// header set (the headers the proxy used to talk to that upstream,
/// which is what gets encoded into the new session id). The header
/// set is `extra_headers` ∪ `Authorization` (when set) ∪
/// `Mcp-Session-Id` (the freshly-minted upstream sid).
///
/// Duplicate URLs in `X-MCP-Servers` are ignored (first-occurrence wins).
/// If any upstream fails to connect, the first such failure is returned
/// as `BadInit::UpstreamConnectFailed` and the remaining in-flight
/// attempts are dropped.
pub async fn connect_all_fresh(
    client: &Client,
    http_headers: &HeaderMap,
) -> Result<Vec<(Connection, IndexMap<String, String>)>, BadInit> {
    let specs = parse_init_headers(http_headers)?;

    let attempts = specs.into_iter().map(|spec| {
        let url = spec.url.clone();
        let authorization = spec.authorization.clone();
        let extra_headers_for_payload = spec.extra_headers.clone();
        async move {
            let conn = client
                .connect(spec.url, spec.authorization, None, spec.extra_headers)
                .await
                .map_err(|source| BadInit::UpstreamConnectFailed {
                    url: url.clone(),
                    source,
                })?;
            let payload_headers = build_canonical_headers(
                &extra_headers_for_payload,
                authorization.as_deref(),
                &conn.session_id,
            );
            Ok::<_, BadInit>((conn, payload_headers))
        }
    });

    try_join_all(attempts).await
}

/// Reconnect to the upstreams encoded in a stale (decoded-but-not-
/// alive) session payload. Each URL gets connected with the headers
/// stored in the payload — the new request's `X-MCP-Servers` /
/// `X-MCP-Headers` / `X-MCP-Authorization` are NOT consulted on this
/// path. The encoded id is the sole source of truth for what to
/// reconnect to and how.
///
/// `Authorization` and `Mcp-Session-Id` are pulled out of each
/// per-URL header map and passed to `Client::connect` as their
/// dedicated arguments; everything else rides as `extra_headers`.
/// The returned pair includes the payload-derived header map, with
/// the `Mcp-Session-Id` refreshed to whatever the upstream returned
/// (which may be the same or a rotated sid).
pub async fn reconnect_from_payload(
    client: &Client,
    payload: &crate::session_manager::SessionPayload,
) -> Result<Vec<(Connection, IndexMap<String, String>)>, BadInit> {
    let attempts = payload.iter().map(|(url, headers)| {
        let url = url.clone();
        let mut headers = headers.clone();
        let session_id = headers.shift_remove(MCP_SESSION_ID_KEY);
        let authorization = headers.shift_remove(AUTHORIZATION_KEY);
        // Everything left is extra_headers.
        let extra = headers;
        async move {
            let conn = client
                .connect(
                    url.clone(),
                    authorization.clone(),
                    session_id,
                    extra.clone(),
                )
                .await
                .map_err(|source| BadInit::UpstreamConnectFailed {
                    url: url.clone(),
                    source,
                })?;
            let canonical =
                build_canonical_headers(&extra, authorization.as_deref(), &conn.session_id);
            Ok::<_, BadInit>((conn, canonical))
        }
    });

    try_join_all(attempts).await
}

/// Build the canonical full header map for one upstream, suitable for
/// encoding into the session id. Sort happens later (in
/// `session_manager::build_payload`); this function just merges.
fn build_canonical_headers(
    extra_headers: &IndexMap<String, String>,
    authorization: Option<&str>,
    upstream_session_id: &str,
) -> IndexMap<String, String> {
    let mut out: IndexMap<String, String> = extra_headers.clone();
    if let Some(auth) = authorization {
        out.insert(AUTHORIZATION_KEY.to_string(), auth.to_string());
    }
    out.insert(MCP_SESSION_ID_KEY.to_string(), upstream_session_id.to_string());
    out
}

fn parse_init_headers(http_headers: &HeaderMap) -> Result<Vec<UpstreamSpec>, BadInit> {
    let servers: Vec<String> = match http_headers.get(SERVERS_HEADER) {
        Some(v) => {
            let s = v.to_str().map_err(|_| BadInit::NotUtf8 { header: SERVERS_HEADER })?;
            serde_json::from_str(s).map_err(|source| BadInit::NotJson {
                header: SERVERS_HEADER,
                source,
            })?
        }
        None => Vec::new(),
    };

    let global_headers: IndexMap<String, String> = match http_headers.get(HEADERS_HEADER) {
        Some(v) => {
            let s = v.to_str().map_err(|_| BadInit::NotUtf8 { header: HEADERS_HEADER })?;
            serde_json::from_str(s).map_err(|source| BadInit::NotJson {
                header: HEADERS_HEADER,
                source,
            })?
        }
        None => IndexMap::new(),
    };

    let per_url_authorization: IndexMap<String, String> =
        match http_headers.get(AUTHORIZATION_HEADER) {
            Some(v) => {
                let s = v.to_str().map_err(|_| BadInit::NotUtf8 {
                    header: AUTHORIZATION_HEADER,
                })?;
                serde_json::from_str(s).map_err(|source| BadInit::NotJson {
                    header: AUTHORIZATION_HEADER,
                    source,
                })?
            }
            None => IndexMap::new(),
        };

    let mut seen = std::collections::HashSet::new();
    let mut specs = Vec::with_capacity(servers.len());
    for url in servers {
        // First-occurrence-wins de-duplication: a duplicate URL is silently
        // ignored. Prevents the proxy from opening N redundant upstream
        // connections to the same server when the client misconfigures.
        if !seen.insert(url.clone()) {
            tracing::debug!(url = %url, "ignoring duplicate X-MCP-Servers entry");
            continue;
        }

        let mut headers = global_headers.clone();
        // Lift Authorization out of the global header set into its dedicated
        // field; per-URL override wins if present.
        let authorization = per_url_authorization
            .get(&url)
            .cloned()
            .or_else(|| headers.shift_remove(AUTHORIZATION_KEY));

        specs.push(UpstreamSpec {
            url,
            authorization,
            extra_headers: headers,
        });
    }

    Ok(specs)
}
