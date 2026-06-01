//! [`NotificationValue`] — the discriminated body of one cli
//! notification, structured as a two-layer enum:
//!
//! - [`NotificationValue::Typed`] holds a [`TypedNotificationValue`]
//!   — an internally-tagged enum (`tag = "type"`) covering every
//!   concrete payload the cli emits. Deserialization tries this
//!   variant first.
//! - [`NotificationValue::Other`] is the untagged catch-all map:
//!   generic emits (`Items<T>`, `Value<V>`), api-call passthroughs
//!   (`Resp`, `Chunk`), and raw `serde_json::Value` payloads. Its
//!   keys flatten directly at the [`super::Notification`] level —
//!   there is no `kind`/`type` envelope, no `value` wrapper.
//!
//! `NotificationValue` itself is `#[serde(untagged)]`. The whole
//! thing then flattens through [`super::Notification`]'s `value`
//! field, so a `Typed::Ok` lands on the wire as
//! `{"type":"ok","ok":true}` and an `Other({"items":[…]})` lands
//! as `{"items":[…]}`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    ActiveAgent, Agent, AgentItems, Cleared, CommandComplete, Detached, Execution,
    Function, Help, Inactive, Installed, Instructions, Inventions, JqResults,
    Laboratory, LogContent, LogStreamReady, Mcp, Me, MessageDelivered,
    MessageQueued, Ok, Pair, Plugin, Plugins, Profile, Published, Schema, Schemas,
    Spawned, State, Swarm, Tool, ToolLine, Tools, Updater, ViewerSendResult,
};

/// One emitted notification payload. Untagged wrapper around
/// [`TypedNotificationValue`] (preferred — has a `type` discriminator)
/// and an `Other` catch-all map.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "cli.output.notification.NotificationValue")]
pub enum NotificationValue {
    #[schemars(title = "Typed")]
    Typed(TypedNotificationValue),
    /// Single catch-all for anything that doesn't get a typed
    /// variant: generic emits (`Items<T>`, `Value<V>`),
    /// api/call.rs passthrough (`Resp`, `Chunk`), and raw
    /// `serde_json::Value`. The map's keys flatten directly into
    /// the surrounding [`super::Notification`] — there is no
    /// `kind`/`type` envelope on the wire.
    ///
    /// Construct via [`NotificationValue::other`]. The payload
    /// must serialize to a JSON object (so its entries can sit at
    /// the [`super::Notification`] level via `#[serde(flatten)]`).
    ///
    /// Wire examples:
    ///   `{"items":[…]}`   (Items<T>)
    ///   `{"value":<V>}`   (Value<V>)
    #[schemars(title = "Other")]
    Other(serde_json::Map<String, serde_json::Value>),
}

/// The typed half of [`NotificationValue`]. Internally tagged with
/// `#[serde(tag = "type")]` — variant discrimination happens on
/// the `type` key.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(rename = "cli.output.notification.TypedNotificationValue")]
pub enum TypedNotificationValue {
    // Agents
    #[schemars(title = "ActiveAgent")]
    ActiveAgent(ActiveAgent),
    #[schemars(title = "Agent")]
    Agent(Agent),
    #[schemars(title = "AgentItems")]
    AgentItems(AgentItems),
    #[schemars(title = "Inactive")]
    Inactive(Inactive),
    #[schemars(title = "MessageDelivered")]
    MessageDelivered(MessageDelivered),
    #[schemars(title = "MessageQueued")]
    MessageQueued(MessageQueued),
    #[schemars(title = "Spawned")]
    Spawned(Spawned),

    // API
    #[schemars(title = "Detached")]
    Detached(Detached),

    // Functions
    #[schemars(title = "Execution")]
    Execution(Execution),
    #[schemars(title = "Function")]
    Function(Function),
    #[schemars(title = "Inventions")]
    Inventions(Inventions),
    #[schemars(title = "Pair")]
    Pair(Pair),
    #[schemars(title = "Profile")]
    Profile(Profile),
    #[schemars(title = "State")]
    State(State),

    // Laboratories
    #[schemars(title = "Laboratory")]
    Laboratory(Laboratory),

    // Swarms
    #[schemars(title = "Swarm")]
    Swarm(Swarm),

    // Shared / multi-command
    #[schemars(title = "Cleared")]
    Cleared(Cleared),
    /// Terminal marker for a plugin-dispatched command. Rides
    /// inside the [`crate::cli::plugins::PluginCommandResponse`]
    /// envelope on the plugin channel — the originating command's
    /// correlation id (when set) is on the envelope's `id` field,
    /// not on this struct. See [`CommandComplete`].
    #[schemars(title = "CommandComplete")]
    CommandComplete(CommandComplete),
    #[schemars(title = "Help")]
    Help(Help),
    #[schemars(title = "Installed")]
    Installed(Installed),
    #[schemars(title = "Instructions")]
    Instructions(Instructions),
    #[schemars(title = "JqResults")]
    JqResults(JqResults),
    #[schemars(title = "LogContent")]
    LogContent(LogContent),
    #[schemars(title = "LogStreamReady")]
    LogStreamReady(LogStreamReady),
    #[schemars(title = "Mcp")]
    Mcp(Mcp),
    #[schemars(title = "Me")]
    Me(Me),
    #[schemars(title = "Ok")]
    Ok(Ok),
    #[schemars(title = "Plugin")]
    Plugin(Plugin),
    /// A notification emitted by a cli plugin and forwarded by the
    /// host. The plugin's payload is nested under `value` as an
    /// arbitrary `serde_json::Value` — objects, strings, numbers,
    /// booleans, arrays, and null are all valid.
    ///
    /// Wire: `{"type":"plugin_notification","value":<any-json>}`.
    #[schemars(title = "PluginNotification")]
    PluginNotification { value: serde_json::Value },
    #[schemars(title = "Plugins")]
    Plugins(Plugins),
    #[schemars(title = "Published")]
    Published(Published),
    #[schemars(title = "Schema")]
    Schema(Schema),
    #[schemars(title = "Schemas")]
    Schemas(Schemas),
    #[schemars(title = "Tool")]
    Tool(Tool),
    #[schemars(title = "ToolLine")]
    ToolLine(ToolLine),
    #[schemars(title = "Tools")]
    Tools(Tools),
    #[schemars(title = "Updater")]
    Updater(Updater),
    #[schemars(title = "ViewerSendResult")]
    ViewerSendResult(ViewerSendResult),
}

impl NotificationValue {
    /// Build an `Other` variant from an arbitrary serializable
    /// payload. Panics if the payload doesn't serialize to a JSON
    /// object — `Other` flattens via `#[serde(flatten)]` on
    /// [`super::Notification::value`], so non-object payloads have
    /// nowhere to land.
    pub fn other<T: Serialize>(value: &T) -> Self {
        let v = serde_json::to_value(value)
            .expect("NotificationValue::other: payload must serialize");
        match v {
            serde_json::Value::Object(map) => Self::Other(map),
            other => panic!(
                "NotificationValue::other: payload must be a JSON object, got {other:?}"
            ),
        }
    }
}

impl From<TypedNotificationValue> for NotificationValue {
    fn from(t: TypedNotificationValue) -> Self {
        Self::Typed(t)
    }
}

macro_rules! from_variant {
    ($($v:ident),* $(,)?) => {
        $(
            impl From<$v> for TypedNotificationValue {
                fn from(v: $v) -> Self { Self::$v(v) }
            }
            impl From<$v> for NotificationValue {
                fn from(v: $v) -> Self {
                    Self::Typed(TypedNotificationValue::$v(v))
                }
            }
        )*
    };
}

from_variant! {
    ActiveAgent, Agent, AgentItems, Inactive, MessageDelivered, MessageQueued, Spawned,
    Detached,
    Execution, Function, Inventions, Pair, Profile, State,
    Laboratory,
    Swarm,
    Cleared, CommandComplete, Help, Installed, Instructions, JqResults, LogContent,
    LogStreamReady, Mcp, Me, Ok, Plugin, Plugins, Published, Schema, Schemas, Tool,
    ToolLine, Tools, Updater, ViewerSendResult,
}
