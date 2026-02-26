use crate::{error, functions, vector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInvention {
    pub id: String,
    pub completions: Vec<super::Completion>,
    pub state: functions::inventions::State,
    pub function: Option<functions::AlphaRemoteFunction>,
    pub created: u64,
    pub object: super::Object,
    pub usage: vector::completions::response::Usage,
    pub error: Option<error::ResponseError>,
}
