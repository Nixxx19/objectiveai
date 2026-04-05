use std::sync::Arc;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub id: String,
    #[serde(flatten)]
    pub inner: objectiveai::error::ResponseError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCompletionCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: Arc<objectiveai::agent::completions::request::AgentCompletionCreateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgentCompletionRequest {
    Begin(AgentCompletionCreateParams),
    Continue(objectiveai::agent::completions::response::streaming::AgentCompletionChunk),
    Error(ResponseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionExecutionCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: Arc<objectiveai::functions::executions::request::FunctionExecutionCreateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionExecutionRequest {
    Begin(FunctionExecutionCreateParams),
    Continue(objectiveai::functions::executions::response::streaming::FunctionExecutionChunk),
    Error(ResponseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInventionRecursiveCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: Arc<objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionInventionRecursiveRequest {
    Begin(FunctionInventionRecursiveCreateParams),
    Continue(objectiveai::functions::inventions::recursive::response::streaming::FunctionInventionRecursiveChunk),
    Error(ResponseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaboratoryExecutionCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: Arc<objectiveai::laboratories::executions::request::LaboratoryExecutionCreateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LaboratoryExecutionRequest {
    Begin(LaboratoryExecutionCreateParams),
    Continue(objectiveai::laboratories::executions::response::streaming::LaboratoryExecutionChunk),
    Error(ResponseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Request {
    AgentCompletion(AgentCompletionRequest),
    FunctionExecution(FunctionExecutionRequest),
    FunctionInventionRecursive(FunctionInventionRecursiveRequest),
    LaboratoryExecution(LaboratoryExecutionRequest),
}
