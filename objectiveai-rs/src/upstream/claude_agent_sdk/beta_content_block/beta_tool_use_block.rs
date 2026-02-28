use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaToolUseBlockType {
    ToolUse,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaToolUseBlock {
    pub id: String,
    pub input: serde_json::Value,
    pub name: String,
    pub r#type: BetaToolUseBlockType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<super::BetaCaller>,
}
