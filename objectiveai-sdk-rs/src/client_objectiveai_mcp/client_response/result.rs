use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Tagged union of response shapes. `Ok` carries no payload — success
/// is implicit in the chunk stream. `Error` carries a numeric `code`
/// and a JSON `message` whose shape mirrors
/// [`crate::error::ResponseError`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "client_objectiveai_mcp.client_response.Result")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Result {
    /// Empty success — the request was accepted.
    Ok,
    /// The request failed. With internally-tagged + struct-variant
    /// serde flattens the inner fields alongside the `type` tag —
    /// e.g. `{"id":"…","type":"error","code":404,"message":…}`.
    Error {
        code: u16,
        message: serde_json::Value,
    },
}
