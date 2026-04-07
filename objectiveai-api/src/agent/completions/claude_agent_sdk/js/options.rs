//! Generates the `const opts = { ... };` JS statement for the Claude Agent SDK subprocess.

use std::sync::Arc;

use super::super::invention_server::InventionServer;
use super::super::mcp_server_config::McpHttpServerConfig;
use super::super::prompt::Prompt;

fn escape_backtick(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('$', "\\$")
}

fn escape_double_quote(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Builds the MCP servers JSON object from connections and an optional invention server.
///
/// Server names are taken from `conn.initialize_result.server_info.name`.
/// If duplicate names exist, both are suffixed with ` ({url})`.
fn build_mcp_servers_json(
    mcp_connections: &[Arc<crate::mcp::Connection>],
    invention_server: Option<&InventionServer>,
) -> String {
    use indexmap::IndexMap;
    use std::collections::HashMap;

    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for conn in mcp_connections {
        let name = &conn.initialize_result.server_info.name;
        *name_counts.entry(name.clone()).or_default() += 1;
    }

    let mut servers: IndexMap<String, serde_json::Value> = IndexMap::new();

    for conn in mcp_connections {
        let name = &conn.initialize_result.server_info.name;
        let key = if name_counts.get(name).copied().unwrap_or(0) > 1 {
            format!("{name} ({})", conn.url)
        } else {
            name.clone()
        };
        let config = McpHttpServerConfig::from(conn.as_ref());
        servers.insert(key, serde_json::to_value(&config).unwrap());
    }

    if let Some(inv) = invention_server {
        let config = inv.mcp_server_config();
        servers.insert(
            "objectiveai-invention".to_string(),
            serde_json::to_value(&config).unwrap(),
        );
    }

    serde_json::to_string(&servers).unwrap()
}

/// Builds the `const opts = { ... };` JS statement with all options defined inline.
pub fn build_options(
    prompt: &Prompt,
    model: &str,
    effort: Option<objectiveai::agent::claude_agent_sdk::Effort>,
    thinking: Option<bool>,
    mcp_connections: &[Arc<crate::mcp::Connection>],
    invention_server: Option<&InventionServer>,
    user_agent: Option<&str>,
) -> String {
    let mcp_servers_json = build_mcp_servers_json(mcp_connections, invention_server);

    let mut fields = Vec::new();
    fields.push("      tools: []".to_string());
    fields.push("      includePartialMessages: true".to_string());
    fields.push("      permissionMode: \"dontAsk\"".to_string());
    fields.push(format!("      model: \"{model}\""));
    fields.push(format!("      mcpServers: {mcp_servers_json}"));

    if let Some(s) = &prompt.system_prompt {
        fields.push(format!("      systemPrompt: `{}`", escape_backtick(s)));
    }
    if let Some(e) = effort {
        fields.push(format!("      effort: \"{}\"", e.as_str()));
    }
    if thinking == Some(false) {
        fields.push("      thinking: { type: 'disabled' }".to_string());
    }
    let session_id = &prompt.message.session_id;
    if !session_id.is_empty() {
        fields.push(format!(
            "      resume: \"{}\"",
            escape_double_quote(session_id)
        ));
    }
    if let Some(ua) = user_agent {
        fields.push(format!(
            "      env: {{ ...process.env, CLAUDE_AGENT_SDK_CLIENT_APP: \"{}\" }}",
            escape_double_quote(ua)
        ));
    }

    format!("    const opts = {{\n{},\n    }};", fields.join(",\n"))
}

#[cfg(test)]
#[path = "options_tests.rs"]
mod tests;
