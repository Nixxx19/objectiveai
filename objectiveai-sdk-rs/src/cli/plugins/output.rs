//! Wire format for the plugin output protocol.
//!
//! Plugins emit one [`PluginOutput`] JSON object per line on their
//! stdout. The host parses each line and dispatches per variant:
//! `error` is displayed, `mcp` announces the URL of an MCP server
//! the plugin just started (dispatched directly to the host's
//! plugin-MCP-begin path), `command` is a request for the host to
//! perform some action and (potentially) reply, and anything that
//! doesn't match those three lands in the untagged `Notification`
//! catch-all and is forwarded to whatever consumer the host has
//! wired up.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use crate::cli::output::{Error, Level, Mcp};

/// One line of plugin output. Untagged outer enum: deserialization
/// tries the three explicit [`TypedPluginOutput`] variants first
/// (`type:"command" | "mcp" | "error"`), and falls through to
/// [`PluginOutput::Notification`] as a catch-all carrying the raw
/// JSON value.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "cli.plugins.PluginOutput")]
pub enum PluginOutput {
    #[schemars(title = "Typed")]
    Typed(TypedPluginOutput),
    /// Final fallback — anything that didn't match a `Typed` variant
    /// lands here as an opaque JSON value. Hosts treat this as a
    /// notification payload to forward upstream.
    #[schemars(title = "Notification")]
    Notification(serde_json::Value),
}

/// The three explicitly-typed plugin output variants. Internally
/// tagged on `type`.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(rename = "cli.plugins.TypedPluginOutput")]
pub enum TypedPluginOutput {
    #[schemars(title = "Command")]
    Command { command: String },
    #[schemars(title = "Mcp")]
    Mcp(Mcp),
    #[schemars(title = "Error")]
    Error(Error),
}
