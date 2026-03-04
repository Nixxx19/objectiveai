use std::collections::HashMap;
use std::sync::Arc;

/// A resolved tool with its display name and origin.
#[derive(Clone)]
pub enum ResolvedTool {
    /// An invention tool provided by ObjectiveAI.
    InventionTool(objectiveai::functions::inventions::InventionTool),
    /// A response-format tool (no extra data needed).
    ResponseFormat,
    /// An MCP tool, with its connection and the original tool name on the server.
    Mcp {
        connection: Arc<crate::mcp::Connection>,
        tool: crate::mcp::tool::Tool,
    },
}

/// Which source a tool originates from, used during conflict resolution.
#[derive(Clone)]
enum ToolSource {
    Mcp {
        connection: Arc<crate::mcp::Connection>,
        tool: crate::mcp::tool::Tool,
        server_name: String,
        url: String,
    },
    Invention(objectiveai::functions::inventions::InventionTool),
    ResponseFormat,
}

/// A tool paired with its origin for name-conflict resolution.
struct SourcedTool {
    name: String,
    source: ToolSource,
}

/// Resolves tool names from MCP connections, invention tools, and an optional
/// response format.
///
/// Returns the list of resolved tool names (in insertion order) and a map from
/// each resolved name to its [`ResolvedTool`].
pub fn resolve_tools(
    mcp_connections: &[Arc<crate::mcp::Connection>],
    mcp_tools: &[Arc<Vec<crate::mcp::tool::Tool>>],
    invention_tools: Option<&[objectiveai::functions::inventions::InventionTool]>,
    response_format: Option<&objectiveai::agent::completions::request::ResponseFormat>,
) -> (Vec<String>, HashMap<String, ResolvedTool>) {
    let mut sourced = Vec::new();

    // MCP tools.
    for (connection, tools) in mcp_connections.iter().zip(mcp_tools.iter()) {
        let server_name = connection.initialize_result.server_info.name.clone();
        let url = connection.url.clone();
        for tool in tools.iter() {
            sourced.push(SourcedTool {
                name: tool.name.clone(),
                source: ToolSource::Mcp {
                    connection: Arc::clone(connection),
                    tool: tool.clone(),
                    server_name: server_name.clone(),
                    url: url.clone(),
                },
            });
        }
    }

    // Invention tools.
    if let Some(inv_tools) = invention_tools {
        for tool in inv_tools {
            sourced.push(SourcedTool {
                name: tool.name.to_string(),
                source: ToolSource::Invention(tool.clone()),
            });
        }
    }

    // Response format tool.
    if let Some(objectiveai::agent::completions::request::ResponseFormat::ToolCall {
        name,
        ..
    }) = response_format
    {
        sourced.push(SourcedTool {
            name: name.clone(),
            source: ToolSource::ResponseFormat,
        });
    }

    resolve_name_conflicts(sourced)
}

/// Resolves name conflicts and returns ordered names + resolved tool map.
///
/// Suffix rules (only applied when there IS a conflict on the same name):
/// - MCP tools get ` (<server_name>)` suffix, unless two MCP tools share both
///   name and server_name, in which case both get ` (<server_name>(<url>))`
/// - Invention tools get ` (objectiveai-invention)` suffix
/// - Response format tool never gets a suffix
fn resolve_name_conflicts(
    sourced: Vec<SourcedTool>,
) -> (Vec<String>, HashMap<String, ResolvedTool>) {
    // Count occurrences of each base name to detect conflicts.
    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for st in &sourced {
        *name_counts.entry(st.name.clone()).or_default() += 1;
    }

    // For conflicting MCP tools, count server_name occurrences per base name.
    let mut mcp_server_name_counts: HashMap<(String, String), usize> = HashMap::new();
    for st in &sourced {
        if name_counts.get(&st.name).copied().unwrap_or(0) > 1 {
            if let ToolSource::Mcp { ref server_name, .. } = st.source {
                *mcp_server_name_counts
                    .entry((st.name.clone(), server_name.clone()))
                    .or_default() += 1;
            }
        }
    }

    let mut names = Vec::with_capacity(sourced.len());
    let mut map = HashMap::with_capacity(sourced.len());

    // Iterate in insertion order (MCP first, then invention, then response format).
    for st in &sourced {
        let has_conflict = name_counts.get(&st.name).copied().unwrap_or(0) > 1;

        let resolved_name = if !has_conflict {
            st.name.clone()
        } else {
            match &st.source {
                ToolSource::Mcp { server_name, url, .. } => {
                    let server_name_duped = mcp_server_name_counts
                        .get(&(st.name.clone(), server_name.clone()))
                        .copied()
                        .unwrap_or(0)
                        > 1;
                    if server_name_duped {
                        format!("{} ({server_name}({url}))", st.name)
                    } else {
                        format!("{} ({server_name})", st.name)
                    }
                }
                ToolSource::Invention(_) => {
                    format!("{} (objectiveai-invention)", st.name)
                }
                ToolSource::ResponseFormat => st.name.clone(),
            }
        };

        let resolved = match &st.source {
            ToolSource::Mcp { connection, tool, .. } => ResolvedTool::Mcp {
                connection: Arc::clone(connection),
                tool: tool.clone(),
            },
            ToolSource::Invention(tool) => ResolvedTool::InventionTool(tool.clone()),
            ToolSource::ResponseFormat => ResolvedTool::ResponseFormat,
        };

        names.push(resolved_name.clone());
        map.insert(resolved_name, resolved);
    }

    (names, map)
}

#[cfg(test)]
#[path = "tool_tests.rs"]
mod tests;
