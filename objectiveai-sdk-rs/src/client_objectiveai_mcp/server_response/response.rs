use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Envelope: correlation `id` + tagged [`super::Result`]. Wire shape:
///
/// ```json
/// {"id":"…","type":"ok","value":{…}}                  // success
/// {"id":"…","type":"error","code":404,"message":…}    // failure
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "client_objectiveai_mcp.server_response.Response")]
pub struct Response {
    /// Matches the `id` of the
    /// [`super::super::server_request::Request`] this response is for.
    pub id: String,
    #[serde(flatten)]
    pub result: super::Result,
}
