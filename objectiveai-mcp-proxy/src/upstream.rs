//! Parsing of the proxy's three custom session-init headers and fan-out
//! connect over the resulting upstream specs.

use std::sync::Arc;

use axum::http::HeaderMap;
use futures::future::join_all;
use indexmap::IndexMap;
use objectiveai::mcp::{Client, Connection};

const SERVERS_HEADER: &str = "X-MCP-Servers";
const HEADERS_HEADER: &str = "X-MCP-Headers";
const AUTHORIZATION_HEADER: &str = "X-MCP-Authorization";
const AUTHORIZATION_KEY: &str = "Authorization";

/// One upstream MCP server the proxy should connect to for a session.
#[derive(Debug)]
pub struct UpstreamSpec {
    pub url: String,
    pub authorization: Option<String>,
    pub extra_headers: IndexMap<String, String>,
}

/// Why parsing the three custom session-init headers failed.
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
}

/// Read `X-MCP-Servers`, `X-MCP-Headers`, `X-MCP-Authorization` from the
/// inbound `initialize` request and produce one [`UpstreamSpec`] per
/// server URL with the merged headers ready to forward.
///
/// All three headers are optional. If `X-MCP-Servers` is absent or empty,
/// returns an empty Vec — the session still initializes, the client just
/// gets nothing back from `tools/list` etc.
///
/// Per-URL precedence: `X-MCP-Headers` is the base set forwarded to every
/// upstream; `X-MCP-Authorization[<url>]` overrides the `Authorization`
/// entry just for that one upstream.
pub fn parse_init_headers(http_headers: &HeaderMap) -> Result<Vec<UpstreamSpec>, BadInit> {
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

    let mut specs = Vec::with_capacity(servers.len());
    for url in servers {
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

/// Connect to every upstream in `specs` in parallel. Failures are logged
/// and dropped; the returned Vec contains only the connections that
/// successfully completed `initialize`. Order matches `specs` order.
pub async fn connect_all(client: &Client, specs: Vec<UpstreamSpec>) -> Vec<Arc<Connection>> {
    let attempts = specs.into_iter().map(|spec| {
        let url = spec.url.clone();
        async move {
            let result = client
                .connect(spec.url, spec.authorization, None, spec.extra_headers)
                .await;
            (url, result)
        }
    });

    let results = join_all(attempts).await;
    let mut connections = Vec::with_capacity(results.len());
    for (url, result) in results {
        match result {
            Ok(conn) => connections.push(conn),
            Err(e) => tracing::warn!(url = %url, error = %e, "upstream connect failed"),
        }
    }
    connections
}
