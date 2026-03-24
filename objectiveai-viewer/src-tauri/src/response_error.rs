use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub id: String,
    #[serde(flatten)]
    pub inner: objectiveai::error::ResponseError,
}
