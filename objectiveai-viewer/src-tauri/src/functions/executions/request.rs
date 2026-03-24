use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionExecutionCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: objectiveai::functions::executions::request::FunctionExecutionCreateParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Request {
    Begin(FunctionExecutionCreateParams),
    Continue(objectiveai::functions::executions::response::streaming::FunctionExecutionChunk),
    Error(crate::response_error::ResponseError),
}
