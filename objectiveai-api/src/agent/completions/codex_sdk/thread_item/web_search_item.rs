use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebSearchItem {
    pub id: String,
    pub query: String,
}
