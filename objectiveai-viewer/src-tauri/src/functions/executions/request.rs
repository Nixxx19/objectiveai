use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Request {
    Begin(objectiveai::functions::executions::request::FunctionExecutionCreateParams),
    Continue(objectiveai::functions::executions::response::streaming::FunctionExecutionChunk),
    Error(objectiveai::error::ResponseError),
}
