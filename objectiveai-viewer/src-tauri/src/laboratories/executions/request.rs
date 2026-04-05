use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaboratoryExecutionCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: objectiveai::laboratories::executions::request::LaboratoryExecutionCreateParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Request {
    Begin(LaboratoryExecutionCreateParams),
    Continue(objectiveai::laboratories::executions::response::streaming::LaboratoryExecutionChunk),
    Error(crate::response_error::ResponseError),
}
