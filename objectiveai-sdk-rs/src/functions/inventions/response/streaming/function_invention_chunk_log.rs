//! On-disk shape of a `FunctionInventionChunk` log file.
//!
//! Mirrors [`super::FunctionInventionChunk`] field-for-field, with
//! `completions: Vec<AgentCompletionChunk>` → `Vec<LogReference>`
//! (each per-agent completion in its own file under
//! `agents/completions/`).

use serde::Serialize;

use crate::agent;
use crate::error;
use crate::filesystem::logs::LogReference;
use crate::functions;

#[derive(Debug, Clone, Serialize)]
pub struct FunctionInventionChunkLog {
    pub id: String,
    pub completions: Vec<LogReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<functions::inventions::State>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<crate::RemotePath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<functions::FullRemoteFunction>,
    pub created: u64,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<agent::completions::response::Usage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<error::ResponseError>,
}
