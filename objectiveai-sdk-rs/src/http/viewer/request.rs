//! Wire shapes for events the viewer client POSTs to a remote viewer's
//! HTTP server. The api server constructs these and pushes them through
//! [`super::Client`]; standalone SDK consumers can do the same.

use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "http.viewer.ResponseError")]
pub struct ResponseError {
    pub id: String,
    #[serde(flatten)]
    pub inner: crate::error::ResponseError,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "http.viewer.AgentCompletionCreateParams")]
pub struct AgentCompletionCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: Arc<crate::agent::completions::request::AgentCompletionCreateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "http.viewer.AgentCompletionRequest")]
#[serde(untagged)]
pub enum AgentCompletionRequest {
    Begin(AgentCompletionCreateParams),
    Continue(crate::agent::completions::response::streaming::AgentCompletionChunk),
    Error(ResponseError),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "http.viewer.FunctionExecutionCreateParams")]
pub struct FunctionExecutionCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: Arc<crate::functions::executions::request::FunctionExecutionCreateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "http.viewer.FunctionExecutionRequest")]
#[serde(untagged)]
pub enum FunctionExecutionRequest {
    Begin(FunctionExecutionCreateParams),
    Continue(crate::functions::executions::response::streaming::FunctionExecutionChunk),
    Error(ResponseError),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "http.viewer.FunctionInventionRecursiveCreateParams")]
pub struct FunctionInventionRecursiveCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: Arc<crate::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "http.viewer.FunctionInventionRecursiveRequest")]
#[serde(untagged)]
pub enum FunctionInventionRecursiveRequest {
    Begin(FunctionInventionRecursiveCreateParams),
    Continue(crate::functions::inventions::recursive::response::streaming::FunctionInventionRecursiveChunk),
    Error(ResponseError),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "http.viewer.LaboratoryExecutionCreateParams")]
pub struct LaboratoryExecutionCreateParams {
    pub id: String,
    #[serde(flatten)]
    pub inner: Arc<crate::laboratories::executions::request::LaboratoryExecutionCreateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "http.viewer.LaboratoryExecutionRequest")]
#[serde(untagged)]
pub enum LaboratoryExecutionRequest {
    Begin(LaboratoryExecutionCreateParams),
    Continue(crate::laboratories::executions::response::streaming::LaboratoryExecutionChunk),
    Error(ResponseError),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "http.viewer.Request")]
#[serde(untagged)]
pub enum Request {
    AgentCompletion(AgentCompletionRequest),
    FunctionExecution(FunctionExecutionRequest),
    FunctionInventionRecursive(FunctionInventionRecursiveRequest),
    LaboratoryExecution(LaboratoryExecutionRequest),
}
