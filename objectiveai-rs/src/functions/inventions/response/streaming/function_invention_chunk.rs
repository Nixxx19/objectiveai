use crate::{error, functions, vector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInventionChunk {
    pub id: String,
    pub completions: Vec<super::CompletionChunk>,
    // yielded after steps with the current state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<functions::inventions::State>,
    // yielded at the end
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<functions::AlphaRemoteFunction>,
    pub created: u64,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<vector::completions::response::Usage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<error::ResponseError>,
}
