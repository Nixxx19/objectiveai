use std::sync::Arc;
use crate::mcp;

#[derive(Debug, Clone)]
pub enum Continuation<OPENROUTER, CLAUDEAGENTSDK, MOCK> {
    Openrouter {
        items: Vec<ContinuationItem<OPENROUTER>>,
        agent: objectiveai::agent::openrouter::Agent,
        mcp_connections: Arc<Vec<Arc<mcp::Connection>>>,
    },
    ClaudeAgentSdk {
        items: Vec<ContinuationItem<CLAUDEAGENTSDK>>,
        agent: objectiveai::agent::claude_agent_sdk::Agent,
        mcp_connections: Arc<Vec<Arc<mcp::Connection>>>,
    },
    Mock {
        items: Vec<ContinuationItem<MOCK>>,
        agent: objectiveai::agent::mock::Agent,
        mcp_connections: Arc<Vec<Arc<mcp::Connection>>>,
    },
}

impl<OPENROUTER, CLAUDEAGENTSDK, MOCK> Continuation<OPENROUTER, CLAUDEAGENTSDK, MOCK> {
    pub fn push_user_message(&mut self, message: objectiveai::agent::completions::message::UserMessage) {
        match self {
            Self::Openrouter { items, .. } => items.push(ContinuationItem::UserMessage(message)),
            Self::ClaudeAgentSdk { items, .. } => items.push(ContinuationItem::UserMessage(message)),
            Self::Mock { items, .. } => items.push(ContinuationItem::UserMessage(message)),
        }
    }

    pub fn push_tool_message(&mut self, message: objectiveai::agent::completions::message::ToolMessage) {
        match self {
            Self::Openrouter { items, .. } => items.push(ContinuationItem::ToolMessage(message)),
            Self::ClaudeAgentSdk { items, .. } => items.push(ContinuationItem::ToolMessage(message)),
            Self::Mock { items, .. } => items.push(ContinuationItem::ToolMessage(message)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ContinuationItem<STATE> {
    State(STATE),
    ToolMessage(objectiveai::agent::completions::message::ToolMessage),
    UserMessage(objectiveai::agent::completions::message::UserMessage),
}
