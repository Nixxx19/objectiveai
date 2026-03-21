use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Chunk {
    FunctionExecution(objectiveai::functions::executions::response::streaming::FunctionExecutionChunk),
    FunctionInventionRecursive(objectiveai::functions::inventions::recursive::response::streaming::FunctionInventionRecursiveChunk),
    Error(objectiveai::error::ResponseError),
}
