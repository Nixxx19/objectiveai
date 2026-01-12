use crate::chat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reasoning {
    pub model: chat::completions::request::Model,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<chat::completions::request::Model>>,
}
