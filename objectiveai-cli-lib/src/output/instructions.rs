use serde::{Deserialize, Serialize};

/// Instructions-ID management output.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Instructions {
    /// Emitted by `<scope> instructions get` and the global `instructions get`.
    Get { id: Option<String>, text: String },
    /// Emitted by `instructions list`.
    List { ids: Vec<String> },
}
