use std::collections::HashMap;
use std::sync::Arc;

/// A resolved tool with its display name and origin.
#[derive(Clone)]
pub enum ResolvedTool {
    /// An invention tool provided by ObjectiveAI.
    InventionTool(objectiveai::functions::inventions::InventionTool),
    /// A response-format tool carrying its description and JSON schema.
    ResponseFormat {
        description: String,
        schema: indexmap::IndexMap<String, serde_json::Value>,
    },
    /// An MCP tool, with its connection and the original tool name on the server.
    Mcp {
        connection: objectiveai::mcp::Connection,
        tool: objectiveai::mcp::tool::Tool,
    },
}

/// Resolves tool names from a single MCP connection (the per-agent proxy
/// connection), invention tools, and an optional response format.
///
/// Returns the list of resolved tool names (in insertion order) and a map
/// from each name to its [`ResolvedTool`]. The proxy is responsible for
/// any cross-upstream name disambiguation it needs to do — at this layer
/// we no longer manufacture suffix-renamed aliases on top.
pub fn resolve_tools(
    mcp_connection: Option<&objectiveai::mcp::Connection>,
    mcp_tools: Option<&Arc<Vec<objectiveai::mcp::tool::Tool>>>,
    invention_tools: Option<&[objectiveai::functions::inventions::InventionTool]>,
    response_format: Option<&objectiveai::agent::completions::request::ResponseFormat>,
) -> (Vec<String>, HashMap<String, ResolvedTool>) {
    let mut names = Vec::new();
    let mut map = HashMap::new();

    if let (Some(connection), Some(tools)) = (mcp_connection, mcp_tools) {
        for tool in tools.iter() {
            names.push(tool.name.clone());
            map.insert(
                tool.name.clone(),
                ResolvedTool::Mcp {
                    connection: connection.clone(),
                    tool: tool.clone(),
                },
            );
        }
    }

    if let Some(inv_tools) = invention_tools {
        for tool in inv_tools {
            names.push(tool.name.to_string());
            map.insert(
                tool.name.to_string(),
                ResolvedTool::InventionTool(tool.clone()),
            );
        }
    }

    if let Some(objectiveai::agent::completions::request::ResponseFormat::ToolCall {
        name,
        description,
        schema,
        ..
    }) = response_format
    {
        names.push(name.clone());
        map.insert(
            name.clone(),
            ResolvedTool::ResponseFormat {
                description: description.clone(),
                schema: schema.clone(),
            },
        );
    }

    (names, map)
}

#[cfg(test)]
#[path = "tool_tests.rs"]
mod tests;
