use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ItemCompletedEvent {
    pub item: super::super::ThreadItem,
}
