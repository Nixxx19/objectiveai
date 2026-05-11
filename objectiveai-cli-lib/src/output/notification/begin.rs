use serde::{Deserialize, Serialize};

/// Emitted by `cli::run` as its very first line, before any other
/// output. A consumer parsing the JSONL stream can use it as a
/// "stream starting" marker.
///
/// Wire: `{"type":"notification","begin":true}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Begin {
    pub begin: bool,
}

pub const BEGIN: Begin = Begin { begin: true };
