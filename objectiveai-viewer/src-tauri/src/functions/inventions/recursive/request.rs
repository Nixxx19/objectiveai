use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInventionRecursiveCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Request {
    Begin(FunctionInventionRecursiveCreateParams),
    Continue(objectiveai::functions::inventions::recursive::response::streaming::FunctionInventionRecursiveChunk),
    Error(crate::response_error::ResponseError),
}
