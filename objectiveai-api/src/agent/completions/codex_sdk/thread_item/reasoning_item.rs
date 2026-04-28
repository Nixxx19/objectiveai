use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReasoningItem {
    pub id: String,
    pub text: String,
}
