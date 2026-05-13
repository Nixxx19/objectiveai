use serde::{Deserialize, Serialize};

/// Result of `<resource> publish`. The SHA identifies the resulting
/// commit on the local filesystem repo.
///
/// Wire: `{"type":"notification","sha":"<commit-sha>"}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Published {
    pub sha: String,
}
