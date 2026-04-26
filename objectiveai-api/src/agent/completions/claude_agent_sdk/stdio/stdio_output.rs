use serde::{Deserialize, Serialize};

use super::id_prefix::IdPrefix;
use super::scanner::scan_id_prefix;
use super::{StdioEndStatus, StdioParseError};

/// One line emitted by the runner on stdout. Always carries an `id`.
///
/// `T` is the per-event payload type — the SDKMessage type the agent
/// completions client deserializes its own JSONL into. `T` is only
/// touched on the [`StdioOutput::Event`] variant; [`StdioOutput::End`]
/// is `T`-free.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StdioOutput<T> {
    /// One SDK message for an in-flight request.
    Event { id: String, event: T },
    /// Terminal line for a request — emitted exactly once per accepted
    /// `run`. The `status` discriminator is flattened into the outer
    /// object on the wire (see [`StdioEndStatus`]).
    End {
        id: String,
        #[serde(flatten)]
        status: StdioEndStatus,
    },
}

impl<T> StdioOutput<T>
where
    T: for<'de> Deserialize<'de>,
{
    /// Parse `line` IFF its `id` field (the second field, after
    /// `type`) equals `request_id`.
    ///
    /// - `Ok(Some(_))` — id matched; full deserialization succeeded.
    /// - `Ok(None)` — id mismatch; the line belongs to another request.
    ///   Caller should keep reading.
    /// - `Err(_)` — line is malformed or has no `id` second field.
    ///   On the wire every [`StdioOutput`] variant requires `id`, so
    ///   a missing-id is a hard error.
    pub fn try_parse(
        line: &str,
        request_id: &str,
    ) -> Result<Option<Self>, StdioParseError> {
        match scan_id_prefix(line)? {
            IdPrefix::Id(id) if id == request_id => {
                let parsed = serde_json::from_str(line)?;
                Ok(Some(parsed))
            }
            IdPrefix::Id(_) => Ok(None),
            IdPrefix::NoId => Err(StdioParseError::MissingIdField),
        }
    }
}
