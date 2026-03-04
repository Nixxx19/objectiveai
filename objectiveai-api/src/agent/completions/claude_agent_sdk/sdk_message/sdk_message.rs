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
