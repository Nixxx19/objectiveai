use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Request {
    Execution(objectiveai::functions::executions::request::FunctionExecutionCreateParams),
    Invention(objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams),
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
