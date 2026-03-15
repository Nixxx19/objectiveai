use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKToolProgressMessageType {
    ToolProgress,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKToolProgressMessage {
    pub r#type: SDKToolProgressMessageType,
    pub tool_use_id: String,
    pub tool_name: String,
    pub parent_tool_use_id: Option<String>,
    pub elapsed_time_seconds: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    pub uuid: String,
    pub session_id: String,
}
