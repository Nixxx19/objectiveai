use serde::{Deserialize, Serialize};

/// Final assistant message text returned by `agents completions create`.
///
/// Wire: `{"type":"notification","content":"...text..."}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    pub content: String,
}
