use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ItemStartedEvent {
    pub item: super::super::ThreadItem,
}
