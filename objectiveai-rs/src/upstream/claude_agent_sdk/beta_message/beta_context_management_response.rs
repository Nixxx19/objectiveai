use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BetaClearToolUsesEditType {
    #[serde(rename = "clear_tool_uses_20250919")]
    ClearToolUses20250919,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaClearToolUses20250919EditResponse {
    pub cleared_input_tokens: i64,
    pub cleared_tool_uses: i64,
    pub r#type: BetaClearToolUsesEditType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BetaClearThinkingEditType {
    #[serde(rename = "clear_thinking_20251015")]
    ClearThinking20251015,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaClearThinking20251015EditResponse {
    pub cleared_input_tokens: i64,
    pub cleared_thinking_turns: i64,
    pub r#type: BetaClearThinkingEditType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaContextManagementEdit {
    ClearToolUses(BetaClearToolUses20250919EditResponse),
    ClearThinking(BetaClearThinking20251015EditResponse),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaContextManagementResponse {
    pub applied_edits: Vec<BetaContextManagementEdit>,
}
