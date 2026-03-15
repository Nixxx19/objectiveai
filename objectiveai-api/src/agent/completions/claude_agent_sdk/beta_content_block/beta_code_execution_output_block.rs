use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaCodeExecutionOutputBlockType {
    CodeExecutionOutput,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCodeExecutionOutputBlock {
    pub file_id: String,
    pub r#type: BetaCodeExecutionOutputBlockType,
}
