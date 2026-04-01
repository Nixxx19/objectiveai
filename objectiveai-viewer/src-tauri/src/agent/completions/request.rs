use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCompletionCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: objectiveai::agent::completions::request::AgentCompletionCreateParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Request {
    Begin(AgentCompletionCreateParams),
    Continue(objectiveai::agent::completions::response::streaming::AgentCompletionChunk),
    Error(crate::response_error::ResponseError),
}
