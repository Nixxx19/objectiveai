use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoListItem {
    pub id: String,
    pub items: Vec<super::TodoItem>,
}
