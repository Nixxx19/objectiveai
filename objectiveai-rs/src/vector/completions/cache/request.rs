use crate::chat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheVoteRequest {
    pub model: chat::completions::request::Model,
    pub models: Option<Vec<chat::completions::request::Model>>,
    pub messages: Vec<chat::completions::request::Message>,
    pub tools: Option<Vec<chat::completions::request::Tool>>,
    pub responses: Vec<chat::completions::request::RichContent>,
}
