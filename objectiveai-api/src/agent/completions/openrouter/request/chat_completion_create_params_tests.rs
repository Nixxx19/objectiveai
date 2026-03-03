//! Tests for [`ChatCompletionCreateParams`] construction.

use std::sync::Arc;

use super::ChatCompletionCreateParams;

/// Runs [`ChatCompletionCreateParams::new_with_tools`] and asserts the result
/// equals `expected`.
fn assert_new_with_tools(
    agent: &objectiveai::agent::openrouter::Agent,
    params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
    messages: &[objectiveai::agent::completions::message::Message],
    mcp_connections: &[Arc<crate::mcp::Connection>],
    mcp_tools: &[Arc<Vec<crate::mcp::tool::Tool>>],
    invention_tools: Option<
        &[objectiveai::functions::inventions::InventionTool],
    >,
    expected: ChatCompletionCreateParams,
) {
    let result = ChatCompletionCreateParams::new_with_tools(
        agent,
        params,
        messages,
        mcp_connections,
        mcp_tools,
        invention_tools,
    );
    assert_eq!(result, expected);
}
