use objectiveai_sdk::mcp;

#[derive(Debug, Clone)]
pub enum Continuation<OPENROUTER, CLAUDEAGENTSDK, CODEXSDK, MOCK> {
    Openrouter {
        items: Vec<ContinuationItem<OPENROUTER>>,
        mcp_connection: Option<mcp::Connection>,
        /// The agent completion's response id (the `id` field on the
        /// response chunk / final completion). Reused across internal
        /// continuation rounds so the composite X-OBJECTIVEAI-AGENT-ID
        /// stays stable for the life of a single agent invocation.
        response_id: String,
    },
    ClaudeAgentSdk {
        items: Vec<ContinuationItem<CLAUDEAGENTSDK>>,
        mcp_connection: Option<mcp::Connection>,
        response_id: String,
    },
    CodexSdk {
        items: Vec<ContinuationItem<CODEXSDK>>,
        mcp_connection: Option<mcp::Connection>,
        response_id: String,
    },
    Mock {
        items: Vec<ContinuationItem<MOCK>>,
        mcp_connection: Option<mcp::Connection>,
        response_id: String,
    },
}

impl<OPENROUTER, CLAUDEAGENTSDK, CODEXSDK, MOCK>
    Continuation<OPENROUTER, CLAUDEAGENTSDK, CODEXSDK, MOCK>
{
    pub fn push_user_message(&mut self, message: objectiveai_sdk::agent::completions::message::UserMessage) {
        match self {
            Self::Openrouter { items, .. } => items.push(ContinuationItem::UserMessage(message)),
            Self::ClaudeAgentSdk { items, .. } => items.push(ContinuationItem::UserMessage(message)),
            Self::CodexSdk { items, .. } => items.push(ContinuationItem::UserMessage(message)),
            Self::Mock { items, .. } => items.push(ContinuationItem::UserMessage(message)),
        }
    }

    pub fn push_tool_message(&mut self, message: objectiveai_sdk::agent::completions::message::ToolMessage) {
        match self {
            Self::Openrouter { items, .. } => items.push(ContinuationItem::ToolMessage(message)),
            Self::ClaudeAgentSdk { items, .. } => items.push(ContinuationItem::ToolMessage(message)),
            Self::CodexSdk { items, .. } => items.push(ContinuationItem::ToolMessage(message)),
            Self::Mock { items, .. } => items.push(ContinuationItem::ToolMessage(message)),
        }
    }

    pub fn upstream(&self) -> objectiveai_sdk::agent::Upstream {
        match self {
            Self::Openrouter { .. } => objectiveai_sdk::agent::Upstream::Openrouter,
            Self::ClaudeAgentSdk { .. } => objectiveai_sdk::agent::Upstream::ClaudeAgentSdk,
            Self::CodexSdk { .. } => objectiveai_sdk::agent::Upstream::CodexSdk,
            Self::Mock { .. } => objectiveai_sdk::agent::Upstream::Mock,
        }
    }

    /// The single MCP proxy connection for this agent (or `None` if the
    /// agent had no MCP servers and no invention tools).
    pub fn mcp_connection(&self) -> Option<&mcp::Connection> {
        match self {
            Self::Openrouter { mcp_connection, .. }
            | Self::ClaudeAgentSdk { mcp_connection, .. }
            | Self::CodexSdk { mcp_connection, .. }
            | Self::Mock { mcp_connection, .. } => mcp_connection.as_ref(),
        }
    }

    /// The agent completion's response id, minted on first entry and
    /// reused across server-side retry rounds. See the corresponding
    /// doc on the enum variants.
    pub fn response_id(&self) -> &str {
        match self {
            Self::Openrouter { response_id, .. }
            | Self::ClaudeAgentSdk { response_id, .. }
            | Self::CodexSdk { response_id, .. }
            | Self::Mock { response_id, .. } => response_id.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ContinuationItem<STATE> {
    State(STATE),
    ToolMessage(objectiveai_sdk::agent::completions::message::ToolMessage),
    UserMessage(objectiveai_sdk::agent::completions::message::UserMessage),
}
