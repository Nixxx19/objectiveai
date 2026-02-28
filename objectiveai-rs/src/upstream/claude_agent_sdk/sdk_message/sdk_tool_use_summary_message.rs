use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKToolUseSummaryMessageType {
    ToolUseSummary,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKToolUseSummaryMessage {
    pub r#type: SDKToolUseSummaryMessageType,
    pub summary: String,
    pub preceding_tool_use_ids: Vec<String>,
    pub uuid: String,
    pub session_id: String,
}
