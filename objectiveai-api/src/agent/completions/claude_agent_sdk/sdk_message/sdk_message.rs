use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SDKMessage {
    AssistantMessage(super::SDKAssistantMessage),
    UserMessage(super::SDKUserMessage),
    UserMessageReplay(super::SDKUserMessageReplay),
    ResultMessage(super::SDKResultMessage),
    SystemMessage(super::SDKSystemMessage),
    PartialAssistantMessage(super::SDKPartialAssistantMessage),
    CompactBoundaryMessage(super::SDKCompactBoundaryMessage),
    StatusMessage(super::SDKStatusMessage),
    HookStartedMessage(super::SDKHookStartedMessage),
    HookProgressMessage(super::SDKHookProgressMessage),
    HookResponseMessage(super::SDKHookResponseMessage),
    ToolProgressMessage(super::SDKToolProgressMessage),
    AuthStatusMessage(super::SDKAuthStatusMessage),
    TaskNotificationMessage(super::SDKTaskNotificationMessage),
    TaskStartedMessage(super::SDKTaskStartedMessage),
    FilesPersistedEvent(super::SDKFilesPersistedEvent),
    ToolUseSummaryMessage(super::SDKToolUseSummaryMessage),
    RateLimitEvent(super::SDKRateLimitEvent),
}

impl SDKMessage {
    /// Returns the session ID if this message variant carries one.
    pub fn session_id(&self) -> Option<&str> {
        match self {
            Self::PartialAssistantMessage(msg) => Some(&msg.session_id),
            Self::ResultMessage(msg) => Some(msg.session_id()),
            Self::UserMessage(msg) => Some(&msg.session_id),
            Self::AssistantMessage(msg) => Some(&msg.session_id),
            _ => None,
        }
    }

    /// Transforms this upstream SDK message into a downstream
    /// [`AgentCompletionChunk`].
    ///
    /// Returns `Some(Ok(chunk))` for messages that produce streaming data,
    /// `Some(Err(Error::RateLimit))` for rate limit events, and `None` for
    /// messages that should be ignored.
    pub fn into_downstream(
        self,
        id: String,
        created: u64,
        agent: String,
        assistant_index: u64,
        is_byok: bool,
        cost_multiplier: rust_decimal::Decimal,
    ) -> Option<
        Result<
            objectiveai::agent::completions::response::streaming::AgentCompletionChunk,
            super::super::Error,
        >,
    > {
        match self {
            Self::PartialAssistantMessage(msg) => {
                msg.into_downstream(id, created, agent, assistant_index).map(Ok)
            }
            Self::UserMessage(msg) => {
                msg.into_downstream(id, created, assistant_index).map(Ok)
            }
            Self::ResultMessage(msg) => {
                Some(Ok(msg.into_downstream(id, created, is_byok, cost_multiplier)))
            }
            Self::RateLimitEvent(_) => Some(Err(super::super::Error::RateLimit)),
            // All other variants are ignored.
            Self::AssistantMessage(_)
            | Self::UserMessageReplay(_)
            | Self::SystemMessage(_)
            | Self::CompactBoundaryMessage(_)
            | Self::StatusMessage(_)
            | Self::HookStartedMessage(_)
            | Self::HookProgressMessage(_)
            | Self::HookResponseMessage(_)
            | Self::ToolProgressMessage(_)
            | Self::AuthStatusMessage(_)
            | Self::TaskNotificationMessage(_)
            | Self::TaskStartedMessage(_)
            | Self::FilesPersistedEvent(_)
            | Self::ToolUseSummaryMessage(_) => None,
        }
    }
}
