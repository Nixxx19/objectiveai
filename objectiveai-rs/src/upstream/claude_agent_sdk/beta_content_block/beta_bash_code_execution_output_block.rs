use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaBashCodeExecutionOutputBlockType {
    BashCodeExecutionOutput,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaBashCodeExecutionOutputBlock {
    pub file_id: String,
    pub r#type: BetaBashCodeExecutionOutputBlockType,
}
