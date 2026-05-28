//! `Id` — a reference to a logged file, expressed as the path
//! relative to the logs directory (same payload as
//! [`crate::filesystem::logs::LogReference::path`]).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// File pointer used throughout the queue-read schema. The string is
/// the path relative to `${config_base_dir}/logs/`, e.g.
/// `"agents/completions/response/messages/reasoning/acc-1_0.json"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(rename = "filesystem.logs.queue.Id")]
pub struct Id(pub String);

impl Id {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }
}

impl From<crate::filesystem::logs::LogReference> for Id {
    fn from(r: crate::filesystem::logs::LogReference) -> Self {
        Self(r.path)
    }
}
