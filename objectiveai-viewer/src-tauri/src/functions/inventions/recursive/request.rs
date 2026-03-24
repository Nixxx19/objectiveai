use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Request {
    Begin(objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams),
    Continue(objectiveai::functions::inventions::recursive::response::streaming::FunctionInventionRecursiveChunk),
    Error(objectiveai::error::ResponseError),
}
