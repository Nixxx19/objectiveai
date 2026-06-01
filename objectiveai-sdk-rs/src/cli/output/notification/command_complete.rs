use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Terminal marker for a plugin-dispatched command. Emitted by the
/// host on the plugin's stdin after the command's `run()` task has
/// resolved. The originating command's correlation id rides at the
/// envelope level (`Handle::request_id`), not as a field on this
/// struct — same correlation surface every preceding response line
/// for the same command carried, so the plugin's read loop demuxes
/// uniformly.
///
/// Wire: `{"type":"notification","type":"command_complete","exit_code":<n>,"request_id":"<id>"}`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[schemars(rename = "cli.output.notification.CommandComplete")]
pub struct CommandComplete {
    /// Exit code the cli command returned. `0` = success, non-zero
    /// = failure (same shape `crate::run::run` returns).
    pub exit_code: i32,
}
