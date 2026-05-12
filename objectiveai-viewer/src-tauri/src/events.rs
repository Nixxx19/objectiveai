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

#[derive(Clone)]
pub enum Event {
    AgentCompletions(agent::completions::request::Request),
    FunctionsExecutions(functions::executions::request::Request),
    FunctionsInventionsRecursive(functions::inventions::recursive::request::Request),
    LaboratoriesExecutions(laboratories::executions::request::Request),
    Plugin(PluginEvent),
}

/// Unified wire shape for every Tauri event the viewer emits to the
/// frontend. `destination` is `"objectiveai"` for built-in events or
/// the plugin's repository name for plugin events. `type` is a
/// snake_case discriminator (built-ins use `agent_completions` etc.;
/// plugins use whatever they declared in their manifest's
/// `viewer_routes[i].type`). `value` is the original request body.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmittedEvent {
    pub destination: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub value: serde_json::Value,
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
    /// Tauri event name (channel) to emit under. All four built-in
    /// variants share the `"objectiveai"` channel; plugin events go
    /// to `plugin-<repo>` so the host bridge can filter per-plugin
    /// and so a plugin named `objectiveai` can't collide with the
    /// root channel.
    pub(crate) fn tauri_event_name(&self) -> std::borrow::Cow<'static, str> {
        match self {
            Event::Plugin(p) => std::borrow::Cow::Owned(format!("plugin-{}", p.plugin)),
            _ => std::borrow::Cow::Borrowed("objectiveai"),
        }
    }

    /// Build the unified [`EmittedEvent`] envelope for this event.
    /// Built-in variants serialize their typed request into the
    /// `value` field; plugin events pass through the route handler's
    /// JSON body verbatim.
    pub(crate) fn to_emitted(&self) -> EmittedEvent {
        match self {
            Event::AgentCompletions(r) => EmittedEvent {
                destination: "objectiveai".to_string(),
                r#type: "agent_completions".to_string(),
                value: serde_json::to_value(r).unwrap_or(serde_json::Value::Null),
            },
            Event::FunctionsExecutions(r) => EmittedEvent {
                destination: "objectiveai".to_string(),
                r#type: "functions_executions".to_string(),
                value: serde_json::to_value(r).unwrap_or(serde_json::Value::Null),
            },
            Event::FunctionsInventionsRecursive(r) => EmittedEvent {
                destination: "objectiveai".to_string(),
                r#type: "functions_inventions_recursive".to_string(),
                value: serde_json::to_value(r).unwrap_or(serde_json::Value::Null),
            },
            Event::LaboratoriesExecutions(r) => EmittedEvent {
                destination: "objectiveai".to_string(),
                r#type: "laboratories_executions".to_string(),
                value: serde_json::to_value(r).unwrap_or(serde_json::Value::Null),
            },
            Event::Plugin(p) => EmittedEvent {
                destination: p.plugin.clone(),
                r#type: p.request.r#type.clone(),
                value: p.request.value.clone(),
            },
        }
    }
}

pub type EventReceiver = mpsc::UnboundedReceiver<Event>;
pub type EventSender = mpsc::UnboundedSender<Event>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tauri_event_name_for_plugin_includes_plugin_name() {
        let e = Event::Plugin(PluginEvent {
            plugin: "myplugin".to_string(),
            request: PluginRequest {
                r#type: "x".to_string(),
                value: serde_json::Value::Null,
            },
        });
        assert_eq!(&*e.tauri_event_name(), "plugin-myplugin");
    }

    #[test]
    fn to_emitted_for_plugin_passes_through_type_and_value() {
        let e = Event::Plugin(PluginEvent {
            plugin: "psyops".to_string(),
            request: PluginRequest {
                r#type: "say".to_string(),
                value: json!({"to":"world"}),
            },
        });
        let em = e.to_emitted();
        assert_eq!(em.destination, "psyops");
        assert_eq!(em.r#type, "say");
        assert_eq!(em.value, json!({"to":"world"}));
    }
}
