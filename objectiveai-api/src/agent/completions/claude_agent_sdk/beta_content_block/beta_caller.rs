use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaDirectCallerType {
    Direct,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BetaDirectCaller {
    pub r#type: BetaDirectCallerType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BetaServerToolCallerType {
    #[serde(rename = "code_execution_20250825")]
    CodeExecution20250825,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaServerToolCaller {
    pub tool_id: String,
    pub r#type: BetaServerToolCallerType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BetaServerToolCaller20260120Type {
    #[serde(rename = "code_execution_20260120")]
    CodeExecution20260120,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaServerToolCaller20260120 {
    pub tool_id: String,
    pub r#type: BetaServerToolCaller20260120Type,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaCaller {
    Direct(BetaDirectCaller),
    ServerTool(BetaServerToolCaller),
    ServerTool20260120(BetaServerToolCaller20260120),
}
