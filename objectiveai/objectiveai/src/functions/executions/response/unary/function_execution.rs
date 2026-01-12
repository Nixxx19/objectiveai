use crate::{
    error,
    functions::{self, executions::response},
    vector,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionExecution {
    pub id: String,
    pub tasks: Vec<super::Task>,
    pub tasks_errors: bool,
    pub reasoning: Option<super::ReasoningSummary>,
    pub output: functions::expression::FunctionOutput,
    pub error: Option<error::ResponseError>,
    pub retry_token: Option<String>,
    pub created: u64,
    pub function: Option<String>,
    pub profile: Option<String>,
    pub object: super::Object,
    pub usage: vector::completions::response::Usage,
}

impl FunctionExecution {
    pub fn any_usage(&self) -> bool {
        self.usage.any_usage()
    }
}

impl From<response::streaming::FunctionExecutionChunk> for FunctionExecution {
    fn from(
        response::streaming::FunctionExecutionChunk {
            id,
            tasks,
            tasks_errors,
            reasoning,
            output,
            error,
            retry_token,
            created,
            function,
            profile,
            object,
            usage,
        }: response::streaming::FunctionExecutionChunk,
    ) -> Self {
        Self {
            id,
            tasks: tasks.into_iter().map(super::Task::from).collect(),
            tasks_errors: tasks_errors.unwrap_or(false),
            reasoning: reasoning.map(super::ReasoningSummary::from),
            output: output.unwrap_or(
                functions::expression::FunctionOutput::Err(
                    serde_json::Value::Null,
                ),
            ),
            error,
            retry_token,
            created,
            function,
            profile,
            object: object.into(),
            usage: usage.unwrap_or_default(),
        }
    }
}
