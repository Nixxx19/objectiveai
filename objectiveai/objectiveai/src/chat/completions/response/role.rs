use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Role {
    #[serde(rename = "assistant")]
    #[default]
    Assistant,
}
