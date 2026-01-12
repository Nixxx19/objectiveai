use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    Instruction, // instructed to output a key
    JsonSchema, // response format json schema containing an enum of possible keys
    ToolCall, // forced tool with argument schema containing an enum of possible keys
}

impl std::default::Default for OutputMode {
    fn default() -> Self {
        OutputMode::Instruction
    }
}
