use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Completion {
    Chat(super::ChatCompletion),
    Tool(super::super::ToolResponse),
}
