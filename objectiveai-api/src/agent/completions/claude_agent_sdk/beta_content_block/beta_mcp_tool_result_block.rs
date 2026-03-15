use serde::{Deserialize, Serialize};

use super::beta_text_block::BetaTextBlock;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaMCPToolResultBlockType {
    McpToolResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaMCPToolResultContent {
    Text(String),
    Blocks(Vec<BetaTextBlock>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaMCPToolResultBlock {
    pub content: BetaMCPToolResultContent,
    pub is_error: bool,
    pub tool_use_id: String,
    pub r#type: BetaMCPToolResultBlockType,
}
