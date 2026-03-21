use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Request {
    FunctionExecution(objectiveai::functions::executions::request::FunctionExecutionCreateParams),
    FunctionInventionRecursive(objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams),
}

#[derive(Parser)]
pub struct Args {
    /// JSON request body (function execution or recursive invention)
    pub request: String,
}

impl Args {
    pub fn parse_request(&self) -> Result<Request, serde_json::Error> {
        serde_json::from_str(&self.request)
    }
}
