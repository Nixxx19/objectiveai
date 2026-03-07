use std::collections::HashMap;
use std::sync::Arc;

use objectiveai::agent::completions::request::{
    AgentCompletionCreateParams, Agent as AgentParam,
};
use objectiveai::agent::claude_agent_sdk::{Agent, AgentBase};

use super::Client;
use crate::agent::completions::upstream_client::UpstreamClient;

fn default_client() -> Client {
    Client::new(None)
}

fn default_agent() -> Agent {
    Agent::try_from(AgentBase {
        model: "test-model".into(),
        ..Default::default()
    })
    .unwrap()
}

fn default_params() -> AgentCompletionCreateParams {
    AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Id("test".into()),
        provider: None,
        agents: None,
        response_format: None,
        seed: None,
        stream: None,
        mcp_server_authorization: None,
    }
}

#[tokio::test]
async fn test_tools_not_allowed_with_tools_present() {
    let client = default_client();
    let agent = default_agent();
    let params = default_params();
    let tool_names = vec!["some_tool".into()];
    let mut tool_map = HashMap::new();
    tool_map.insert(
        "some_tool".into(),
        crate::agent::completions::tool::ResolvedTool::ResponseFormat {
            description: "test".into(),
            schema: indexmap::IndexMap::new(),
        },
    );

    let result = client
        .create(
            "test", 1000, &agent, &params, &[], &[], None,
            &tool_names, &tool_map, None, None,
            rust_decimal::Decimal::ONE, false,
        )
        .await;
    match result {
        Err(super::Error::ToolsNotAllowed) => {}
        Err(e) => panic!("expected ToolsNotAllowed, got {e}"),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn test_tools_not_allowed_without_tools_proceeds() {
    let client = default_client();
    let agent = default_agent();
    let params = default_params();

    // With no tools, tools_enabled = false should not cause ToolsNotAllowed.
    // It will fail for other reasons (no SDK installed), but NOT with ToolsNotAllowed.
    let result = client
        .create(
            "test", 1000, &agent, &params, &[], &[], None,
            &[], &HashMap::new(), None, None,
            rust_decimal::Decimal::ONE, false,
        )
        .await;
    match result {
        Err(super::Error::ToolsNotAllowed) => {
            panic!("should not get ToolsNotAllowed when no tools are present")
        }
        _ => {} // any other result is fine
    }
}
