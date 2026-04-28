use serde::{Deserialize, Serialize};

/// Top-level fatal error from the event stream. Distinct from
/// [`super::TurnFailedEvent`]: that one signals a turn that ran but failed,
/// while this one signals the stream itself terminated abnormally.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThreadErrorEvent {
    pub message: String,
}
