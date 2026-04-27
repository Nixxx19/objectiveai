//! Tool execution properties.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The tool's preference for task-augmented execution.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[schemars(rename = "mcp.tool.TaskSupport")]
pub enum TaskSupport {
    /// Clients MUST invoke the tool as a task.
    Required,
    /// Clients MAY invoke the tool as a task or normal request.
    Optional,
    /// Clients MUST NOT attempt to invoke the tool as a task.
    Forbidden,
}

/// Execution-related properties for a tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "mcp.tool.ToolExecution")]
pub struct ToolExecution {
    /// Indicates the tool's preference for task-augmented execution.
    /// Defaults to "forbidden" if not present.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "taskSupport")]
    pub task_support: Option<TaskSupport>,
}
