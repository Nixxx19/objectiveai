use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaMCPToolUseBlockType {
    McpToolUse,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaMCPToolUseBlock {
    pub id: String,
    pub input: serde_json::Value,
    pub name: String,
    pub server_name: String,
    pub r#type: BetaMCPToolUseBlockType,
}
