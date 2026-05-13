use serde::{Deserialize, Serialize};

/// Number of log files cleared by `<scope> logs clear` (or the global
/// `logs clear`). Wire: `{"type":"notification","cleared":7}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cleared {
    pub cleared: u64,
}
