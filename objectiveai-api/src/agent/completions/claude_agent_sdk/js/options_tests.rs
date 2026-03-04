use super::super::super::prompt::Prompt;
use super::super::super::sdk_message::*;
use super::build_options;

fn prompt(system_prompt: Option<&str>, session_id: &str) -> Prompt {
    Prompt {
        system_prompt: system_prompt.map(|s| s.to_string()),
        message: SDKUserMessage {
            r#type: SDKUserMessageType::User,
            message: MessageParam {
                content: MessageParamContent::String("test".to_string()),
                role: MessageParamRole::User,
            },
            parent_tool_use_id: None,
            is_synthetic: None,
            tool_use_result: None,
            uuid: None,
            session_id: session_id.to_string(),
        },
    }
}

#[test]
fn test_minimal() {
    let p = prompt(None, "");
    assert_eq!(
        build_options(&p, "claude-sonnet-4-20250514", None, None, &[], None, None),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"claude-sonnet-4-20250514\",\n      mcpServers: {},\n    };",
    );
}

#[test]
fn test_with_system_prompt() {
    let p = prompt(Some("You are helpful"), "");
    assert_eq!(
        build_options(&p, "claude-sonnet-4-20250514", None, None, &[], None, None),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"claude-sonnet-4-20250514\",\n      mcpServers: {},\n      systemPrompt: `You are helpful`,\n    };",
    );
}

#[test]
fn test_system_prompt_with_backticks() {
    let p = prompt(Some("Use `code` and $var"), "");
    assert_eq!(
        build_options(&p, "claude-sonnet-4-20250514", None, None, &[], None, None),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"claude-sonnet-4-20250514\",\n      mcpServers: {},\n      systemPrompt: `Use \\`code\\` and \\$var`,\n    };",
    );
}

#[test]
fn test_system_prompt_with_backslash() {
    let p = prompt(Some("path\\to\\file"), "");
    assert_eq!(
        build_options(&p, "claude-sonnet-4-20250514", None, None, &[], None, None),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"claude-sonnet-4-20250514\",\n      mcpServers: {},\n      systemPrompt: `path\\\\to\\\\file`,\n    };",
    );
}

#[test]
fn test_effort_low() {
    let p = prompt(None, "");
    assert_eq!(
        build_options(
            &p,
            "claude-sonnet-4-20250514",
            Some(objectiveai::agent::claude_agent_sdk::Effort::Low),
            None,
            &[],
            None,
            None,
        ),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"claude-sonnet-4-20250514\",\n      mcpServers: {},\n      effort: \"low\",\n    };",
    );
}

#[test]
fn test_effort_max() {
    let p = prompt(None, "");
    assert_eq!(
        build_options(
            &p,
            "claude-sonnet-4-20250514",
            Some(objectiveai::agent::claude_agent_sdk::Effort::Max),
            None,
            &[],
            None,
            None,
        ),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"claude-sonnet-4-20250514\",\n      mcpServers: {},\n      effort: \"max\",\n    };",
    );
}

#[test]
fn test_thinking_disabled() {
    let p = prompt(None, "");
    assert_eq!(
        build_options(&p, "claude-sonnet-4-20250514", None, Some(false), &[], None, None),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"claude-sonnet-4-20250514\",\n      mcpServers: {},\n      thinking: { type: 'disabled' },\n    };",
    );
}

#[test]
fn test_thinking_true_is_noop() {
    let p = prompt(None, "");
    // thinking: Some(true) should NOT add thinking field
    assert_eq!(
        build_options(&p, "claude-sonnet-4-20250514", None, Some(true), &[], None, None),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"claude-sonnet-4-20250514\",\n      mcpServers: {},\n    };",
    );
}

#[test]
fn test_with_session_id() {
    let p = prompt(None, "sess-abc-123");
    assert_eq!(
        build_options(&p, "claude-sonnet-4-20250514", None, None, &[], None, None),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"claude-sonnet-4-20250514\",\n      mcpServers: {},\n      resume: \"sess-abc-123\",\n    };",
    );
}

#[test]
fn test_session_id_with_quotes() {
    let p = prompt(None, "sess\"special");
    assert_eq!(
        build_options(&p, "claude-sonnet-4-20250514", None, None, &[], None, None),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"claude-sonnet-4-20250514\",\n      mcpServers: {},\n      resume: \"sess\\\"special\",\n    };",
    );
}

#[test]
fn test_with_user_agent() {
    let p = prompt(None, "");
    assert_eq!(
        build_options(
            &p,
            "claude-sonnet-4-20250514",
            None,
            None,
            &[],
            None,
            Some("objectiveai/1.0"),
        ),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"claude-sonnet-4-20250514\",\n      mcpServers: {},\n      env: { ...process.env, CLAUDE_AGENT_SDK_CLIENT_APP: \"objectiveai/1.0\" },\n    };",
    );
}

#[test]
fn test_user_agent_with_quotes() {
    let p = prompt(None, "");
    assert_eq!(
        build_options(
            &p,
            "claude-sonnet-4-20250514",
            None,
            None,
            &[],
            None,
            Some("my\"agent"),
        ),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"claude-sonnet-4-20250514\",\n      mcpServers: {},\n      env: { ...process.env, CLAUDE_AGENT_SDK_CLIENT_APP: \"my\\\"agent\" },\n    };",
    );
}

#[test]
fn test_all_options_combined() {
    let p = prompt(Some("Be concise"), "sess-42");
    assert_eq!(
        build_options(
            &p,
            "claude-opus-4-20250514",
            Some(objectiveai::agent::claude_agent_sdk::Effort::High),
            Some(false),
            &[],
            None,
            Some("myapp/2.0"),
        ),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"claude-opus-4-20250514\",\n      mcpServers: {},\n      systemPrompt: `Be concise`,\n      effort: \"high\",\n      thinking: { type: 'disabled' },\n      resume: \"sess-42\",\n      env: { ...process.env, CLAUDE_AGENT_SDK_CLIENT_APP: \"myapp/2.0\" },\n    };",
    );
}

#[test]
fn test_different_model() {
    let p = prompt(None, "");
    assert_eq!(
        build_options(&p, "openai/gpt-4o", None, None, &[], None, None),
        "    const opts = {\n      tools: [],\n      includePartialMessages: true,\n      permissionMode: \"dontAsk\",\n      model: \"openai/gpt-4o\",\n      mcpServers: {},\n    };",
    );
}
