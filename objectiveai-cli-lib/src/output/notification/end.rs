use serde::{Deserialize, Serialize};

/// Emitted by `cli::run` as its very last line, after every other
/// output (including any error and exit-code-determining emit). A
/// consumer parsing the JSONL stream can use it as a "stream finished"
/// marker — once seen, no more notifications will arrive.
///
/// Wire: `{"type":"notification","end":true}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct End {
    pub end: bool,
}

pub const END: End = End { end: true };
