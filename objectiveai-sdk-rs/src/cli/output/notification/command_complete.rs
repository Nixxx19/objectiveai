use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Terminal marker for a plugin-dispatched command. Emitted by the
/// host on the plugin's stdin after the command's `run()` task has
/// resolved, wrapped in the same
/// [`crate::cli::plugins::PluginCommandResponse`] envelope every
/// preceding response line for the same command rode through — the
/// correlation id (when the originating `TypedPluginOutput::Command`
/// had one) is on the envelope's `id` field, not on this struct, so
/// the plugin's read loop demuxes uniformly.
///
/// Wire (inside the envelope, with id):
/// `{"id":"<id>","value":{"type":"notification","type":"command_complete","exit_code":<n>}}`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[schemars(rename = "cli.output.notification.CommandComplete")]
pub struct CommandComplete {
    /// Exit code the cli command returned. `0` = success, non-zero
    /// = failure (same shape `crate::run::run` returns).
    pub exit_code: i32,
}
