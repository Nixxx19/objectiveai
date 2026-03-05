use std::sync::Arc;
use crate::mcp;

#[derive(Debug, Clone)]
pub enum Continuation<OPENROUTER, CLAUDEAGENTSDK, MOCK> {
    Openrouter {
        items: Vec<ContinuationItem<OPENROUTER>>,
        agent: objectiveai::agent::openrouter::Agent,
        mcp_connections: Vec<Arc<mcp::Connection>>,
    },
    ClaudeAgentSdk {
        items: Vec<ContinuationItem<CLAUDEAGENTSDK>>,
        agent: objectiveai::agent::claude_agent_sdk::Agent,
        mcp_connections: Vec<Arc<mcp::Connection>>,
    },
    Mock {
        items: Vec<ContinuationItem<MOCK>>,
        agent: objectiveai::agent::mock::Agent,
        mcp_connections: Vec<Arc<mcp::Connection>>,
    },
}

#[derive(Debug, Clone)]
pub enum ContinuationItem<STATE> {
    State(STATE),
    ToolMessage(objectiveai::agent::completions::message::ToolMessage),
    UserMessage(objectiveai::agent::completions::message::UserMessage),
}
