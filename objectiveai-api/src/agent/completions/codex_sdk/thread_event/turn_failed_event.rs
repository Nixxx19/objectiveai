use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnFailedEvent {
    pub error: super::ThreadError,
}
