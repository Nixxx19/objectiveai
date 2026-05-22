use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Envelope: correlation `id` + tagged [`super::Payload`]. Wire shape
/// (the `id` field lives at the envelope level, the `type`
/// discriminator and the variant's payload fields are flattened
/// alongside):
///
/// ```json
/// {"id":"…","type":"mcp_tools_list","cursor":"…"}
/// {"id":"…","type":"mcp_tools_call","name":"…","arguments":{…}}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "client_objectiveai_mcp.server_request.Request")]
pub struct Request {
    /// Server-minted correlation id. Echoed by the matching
    /// [`super::super::server_response::Response`].
    pub id: String,
    #[serde(flatten)]
    pub payload: super::Payload,
}
