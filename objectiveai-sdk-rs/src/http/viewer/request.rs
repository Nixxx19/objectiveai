//! Wire shapes for events the viewer client POSTs to a remote viewer's
//! HTTP server. The api server constructs these and pushes them through
//! [`super::Client`]; standalone SDK consumers can do the same.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub id: String,
    #[serde(flatten)]
    pub inner: crate::error::ResponseError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCompletionCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: Arc<crate::agent::completions::request::AgentCompletionCreateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgentCompletionRequest {
    Begin(AgentCompletionCreateParams),
    Continue(crate::agent::completions::response::streaming::AgentCompletionChunk),
    Error(ResponseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionExecutionCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: Arc<crate::functions::executions::request::FunctionExecutionCreateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionExecutionRequest {
    Begin(FunctionExecutionCreateParams),
    Continue(crate::functions::executions::response::streaming::FunctionExecutionChunk),
    Error(ResponseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInventionRecursiveCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: Arc<crate::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionInventionRecursiveRequest {
    Begin(FunctionInventionRecursiveCreateParams),
    Continue(crate::functions::inventions::recursive::response::streaming::FunctionInventionRecursiveChunk),
    Error(ResponseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaboratoryExecutionCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: Arc<crate::laboratories::executions::request::LaboratoryExecutionCreateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LaboratoryExecutionRequest {
    Begin(LaboratoryExecutionCreateParams),
    Continue(crate::laboratories::executions::response::streaming::LaboratoryExecutionChunk),
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
