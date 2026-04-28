//! Streamable-HTTP response parsing.
//!
//! The MCP wire spec lets a server respond to a POSTed JSON-RPC request
//! with either a bare JSON body (`Content-Type: application/json`) or an
//! SSE envelope (`Content-Type: text/event-stream`) wrapping a single
//! `data:` line whose payload is the same JSON-RPC response. Real-world
//! servers vary — `rmcp`'s `StreamableHttpService` always uses SSE, while
//! many production MCP servers reply with bare JSON. Clients have to
//! tolerate both.

/// Parses a JSON-RPC response from a streamable-HTTP `Response`. Accepts
/// either a bare JSON body or an SSE envelope; in the SSE case every
/// `data:` line is concatenated and parsed as a single JSON document.
pub(crate) async fn parse_streamable_http_response<T: serde::de::DeserializeOwned>(
    url: &str,
    response: reqwest::Response,
) -> Result<T, super::Error> {
    let bytes = response.bytes().await.map_err(|source| super::Error::Request {
        url: url.to_string(),
        source,
    })?;
    if let Ok(v) = serde_json::from_slice::<T>(&bytes) {
        return Ok(v);
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        super::Error::MalformedResponse {
            url: url.to_string(),
            message: "response body is not valid UTF-8".into(),
        }
    })?;
    let collected: String = text
        .lines()
        .filter_map(|l| l.strip_prefix("data: ").or_else(|| l.strip_prefix("data:")))
        .collect();
    serde_json::from_str(&collected).map_err(|e| {
        super::Error::MalformedResponse {
            url: url.to_string(),
            message: format!(
                "neither JSON nor SSE-wrapped JSON: {e}; body starts with: {}",
                text.chars().take(200).collect::<String>(),
            ),
        }
    })
}
