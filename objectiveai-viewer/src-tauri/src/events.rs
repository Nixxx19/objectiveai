//! Event bus types shared by axum handlers, the mpsc channel that
//! buffers events while the React frontend boots, and Tauri's
//! `Emitter` that fans events out to the renderer.
//!
//! Each `Event` variant maps to a unique Tauri event name via
//! [`Event::name`]; plugin events use a per-plugin name
//! (`plugin-<name>`) so the host-side bridge can filter and forward
//! to the matching iframe.

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::agent;
use crate::functions;
use crate::laboratories;

#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum Event {
    AgentCompletions(agent::completions::request::Request),
    FunctionsExecutions(functions::executions::request::Request),
    FunctionsInventionsRecursive(functions::inventions::recursive::request::Request),
    LaboratoriesExecutions(laboratories::executions::request::Request),
    Plugin(PluginEvent),
}

/// Payload emitted whenever a plugin's viewer route is hit. `plugin`
/// identifies which plugin's iframe should receive the event; the
/// host's tab shell uses this to route via postMessage. `request`
/// wraps the route's manifest-declared `type` tag and the JSON body.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginEvent {
    pub plugin: String,
    pub request: PluginRequest,
}

/// Wire shape forwarded to a plugin's iframe. `type` is the string
/// tag the plugin author declared in their manifest's `viewer_routes`
/// entry; `value` is the JSON body of the HTTP request (or
/// `Value::Null` for bodies axum couldn't parse / GET requests).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginRequest {
    #[serde(rename = "type")]
    pub r#type: String,
    pub value: serde_json::Value,
}

impl Event {
    /// Tauri event name to emit this event under. Static for the
    /// built-in variants; `plugin-<name>` for plugin events so the
    /// host bridge can filter per-plugin.
    pub(crate) fn name(&self) -> std::borrow::Cow<'static, str> {
        match self {
            Event::AgentCompletions(_) => std::borrow::Cow::Borrowed("agent-completions"),
            Event::FunctionsExecutions(_) => std::borrow::Cow::Borrowed("functions-executions"),
            Event::FunctionsInventionsRecursive(_) => {
                std::borrow::Cow::Borrowed("functions-inventions-recursive")
            }
            Event::LaboratoriesExecutions(_) => {
                std::borrow::Cow::Borrowed("laboratories-executions")
            }
            Event::Plugin(p) => std::borrow::Cow::Owned(format!("plugin-{}", p.plugin)),
        }
    }
}

pub type EventReceiver = mpsc::UnboundedReceiver<Event>;
pub type EventSender = mpsc::UnboundedSender<Event>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_name_for_plugin_includes_plugin_name() {
        let e = Event::Plugin(PluginEvent {
            plugin: "myplugin".to_string(),
            request: PluginRequest {
                r#type: "x".to_string(),
                value: serde_json::Value::Null,
            },
        });
        assert_eq!(&*e.name(), "plugin-myplugin");
    }
}
