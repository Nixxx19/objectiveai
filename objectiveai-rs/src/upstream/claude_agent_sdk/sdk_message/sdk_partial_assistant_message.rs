use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKPartialAssistantMessageType {
    StreamEvent,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKPartialAssistantMessage {
    pub r#type: SDKPartialAssistantMessageType,
    pub event: super::super::beta_raw_message_stream_event::BetaRawMessageStreamEvent,
    pub parent_tool_use_id: Option<String>,
    pub uuid: String,
    pub session_id: String,
}
