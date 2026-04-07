//! Inline JavaScript generation for the Claude Agent SDK subprocess (agent completions).

use std::sync::Arc;

use super::super::invention_server::InventionServer;
use super::super::prompt::Prompt;

/// Builds inline Node.js code that invokes the Claude Agent SDK `query()` function
/// for agent completions.
///
/// The generated script:
/// 1. Deletes `CLAUDECODE` env var to avoid conflicts
/// 2. Creates an async generator yielding the SDK user message
/// 3. Configures query options (model, system prompt, effort, thinking, MCP servers)
/// 4. Optionally resumes a session via `opts.resume`
/// 5. Streams events to stdout as JSONL
pub fn build_js(
    prompt: &Prompt,
    model: &str,
    effort: Option<objectiveai::agent::claude_agent_sdk::Effort>,
    thinking: Option<bool>,
    mcp_connections: &[Arc<crate::mcp::Connection>],
    invention_server: Option<&InventionServer>,
    user_agent: Option<&str>,
) -> Result<String, super::super::Error> {
    let message_js = super::build_message(prompt)?;
    let options_js = super::build_options(
        prompt,
        model,
        effort,
        thinking,
        mcp_connections,
        invention_server,
        user_agent,
    );

    Ok(format!(
        r#"
delete process.env.CLAUDECODE;
const {{ query }} = require(process.env.CLAUDE_AGENT_SDK_PATH || "@anthropic-ai/claude-agent-sdk");

(async () => {{
  try {{
{message_js}

    async function* messages() {{
      yield message;
    }}

{options_js}

    const stream = query({{ prompt: messages(), options: opts }});

    for await (const event of stream) {{
      process.stdout.write(JSON.stringify(event) + "\n");
    }}
  }} catch (e) {{
    process.stderr.write(e.message || String(e));
    process.exit(1);
  }}
}})();
"#
    ))
}
