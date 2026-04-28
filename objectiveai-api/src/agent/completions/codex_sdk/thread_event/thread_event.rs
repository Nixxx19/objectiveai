use serde::{Deserialize, Serialize};

/// Top-level JSONL events emitted by `codex exec`. The wire form is
/// `{"type": <discriminator>, ...payload}`. The discriminator strings come
/// straight from the Python SDK's `_EVENT_MODELS` registry in `parsing.py`.
///
/// Note the dotted variants (`thread.started`, `turn.started`, etc.) — those
/// require explicit `#[serde(rename = "...")]` because `rename_all` cannot
/// introduce dots.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ThreadEvent {
    #[serde(rename = "thread.started")]
    ThreadStarted(super::ThreadStartedEvent),
    #[serde(rename = "turn.started")]
    TurnStarted(super::TurnStartedEvent),
    #[serde(rename = "turn.completed")]
    TurnCompleted(super::TurnCompletedEvent),
    #[serde(rename = "turn.failed")]
    TurnFailed(super::TurnFailedEvent),
    #[serde(rename = "item.started")]
    ItemStarted(super::ItemStartedEvent),
    #[serde(rename = "item.updated")]
    ItemUpdated(super::ItemUpdatedEvent),
    #[serde(rename = "item.completed")]
    ItemCompleted(super::ItemCompletedEvent),
    #[serde(rename = "error")]
    Error(super::ThreadErrorEvent),
}
