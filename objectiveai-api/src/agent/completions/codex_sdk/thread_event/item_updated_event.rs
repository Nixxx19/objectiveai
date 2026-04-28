use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ItemUpdatedEvent {
    pub item: super::super::ThreadItem,
}
