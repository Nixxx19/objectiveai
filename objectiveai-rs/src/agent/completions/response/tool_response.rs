use crate::agent;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "agent.completions.response.ToolResponse")]
pub struct ToolResponse {
    pub role: ToolRole,
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub index: u64,
    #[serde(flatten)]
    pub inner: agent::completions::message::ToolMessage,
}

impl ToolResponse {
    /// Produces log files for this tool message.
    ///
    /// Returns `(reference, files)` where `reference` is a
    /// `{"type": "reference", "path": ...}` JSON value, and `files`
    /// contains all produced files including the message itself and
    /// extracted media.
    #[cfg(feature = "filesystem")]
    pub fn produce_files(&self, id: &str, prefix: &str) -> (serde_json::Value, Vec<(String, Vec<u8>)>) {
        let stem = format!("{id}_{}", self.index);
        let path = format!("{prefix}messages/{stem}.json");
        let mut files = Vec::new();

        // Serialize a shell without content to avoid double-serialization
        let shell = ToolResponse {
            role: self.role,
            index: self.index,
            inner: agent::completions::message::ToolMessage {
                content: agent::completions::message::RichContent::Text(String::new()),
                tool_call_id: self.inner.tool_call_id.clone(),
            },
        };
        let mut msg_json = serde_json::to_value(&shell).unwrap();

        // Extract media from content (flattened, so "content" is at root)
        let mut content = self.inner.content.clone();
        content.prepare();
        let (content_json, media_files) = content.extract_media(prefix, &stem);
        msg_json["content"] = content_json;
        files.extend(media_files);

        files.push((path.clone(), serde_json::to_vec_pretty(&msg_json).unwrap()));

        (serde_json::json!({ "type": "reference", "path": path }), files)
    }
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, JsonSchema, arbitrary::Arbitrary,
)]
#[schemars(rename = "agent.completions.response.ToolRole")]
pub enum ToolRole {
    #[serde(rename = "tool")]
    #[default]
    Tool,
}
