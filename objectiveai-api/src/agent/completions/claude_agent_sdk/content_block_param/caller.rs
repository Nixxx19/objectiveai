use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DirectCallerType {
    Direct,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectCaller {
    pub r#type: DirectCallerType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerToolCallerType {
    #[serde(rename = "code_execution_20250825")]
    CodeExecution20250825,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ServerToolCaller {
    pub tool_id: String,
    pub r#type: ServerToolCallerType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerToolCaller20260120Type {
    #[serde(rename = "code_execution_20260120")]
    CodeExecution20260120,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ServerToolCaller20260120 {
    pub tool_id: String,
    pub r#type: ServerToolCaller20260120Type,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Caller {
    Direct(DirectCaller),
    ServerTool(ServerToolCaller),
    ServerTool20260120(ServerToolCaller20260120),
}
