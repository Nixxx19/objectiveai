//! [`NotificationValue`] — discriminated enum of every
//! notification payload the cli emits.
//!
//! Lives one level inside [`super::Notification`]'s `value` field,
//! so the wire shape is
//! `{"type":"notification","value":{"kind":"<variant>",<fields>}}`.
//! The `kind` discriminator was added so a downstream consumer can
//! do a single `serde_json::from_str::<Output>(line)` and dispatch
//! on the variant without already knowing which payload to expect.
//!
//! Every concrete struct the cli emits gets a typed variant.
//! Generic and one-off payloads (`Items<T>`, `Value<V>`, raw
//! `serde_json::Value`, api-call passthrough `Resp`/`Chunk`) route
//! through the single [`NotificationValue::Other`] catch-all, which
//! is a `serde_json::Map` that flattens directly alongside `kind`
//! — no inner field wrapper.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    ActiveAgent, Agent, AgentItems, Cleared, Detached, Execution, Function, Help,
    Inactive, Installed, Instructions, Inventions, JqResults, Laboratory, LogContent,
    LogStreamReady, Mcp, Me, MessageDelivered, MessageQueued, Ok, Pair, Plugin,
    Plugins, Profile, Published, Schema, Schemas, Spawned, State, Swarm, Tool,
    ToolLine, Tools, Updater, ViewerSendResult,
};

/// One emitted notification payload. The `kind` tag discriminates
/// the variant. See module-level docs for the wire shape.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[schemars(rename = "cli.output.notification.NotificationValue")]
pub enum NotificationValue {
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
    /// booleans, arrays, and null are all valid. Replaces the old
    /// `NotificationValue::other` flatten-onto-`kind` shape (which
    /// collided with payloads that had their own `"kind"` key and
    /// panicked on non-object inputs).
    ///
    /// Wire: `{"kind":"plugin_notification","value":<any-json>}`.
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

    /// Single catch-all for anything that doesn't get a typed
    /// variant: generic emits (`Items<T>`, `Value<V>`),
    /// api/call.rs passthrough (`Resp`, `Chunk`), and raw
    /// `serde_json::Value`. The map's keys flatten directly
    /// alongside `kind` — there is no inner field wrapping.
    ///
    /// Construct via [`NotificationValue::other`]. The payload
    /// must serialize to a JSON object (so its entries can sit at
    /// the same level as `kind`), and its keys cannot include
    /// `"kind"` (would collide with the discriminator).
    ///
    /// Wire examples:
    ///   `{"kind":"other","items":[…]}`        (Items<T>)
    ///   `{"kind":"other","value":<V>}`        (Value<V>)
    #[schemars(title = "Other")]
    Other(serde_json::Map<String, serde_json::Value>),
}

impl NotificationValue {
    /// Build an `Other` variant from an arbitrary serializable
    /// payload. Panics if the payload doesn't serialize to a JSON
    /// object — `Other` flattens, so non-object payloads have
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

macro_rules! from_variant {
    ($($v:ident),* $(,)?) => {
        $(
            impl From<$v> for NotificationValue {
                fn from(v: $v) -> Self { Self::$v(v) }
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
    Cleared, Help, Installed, Instructions, JqResults, LogContent, LogStreamReady,
    Mcp, Me, Ok, Plugin, Plugins, Published, Schema, Schemas, Tool, ToolLine, Tools,
    Updater, ViewerSendResult,
}
