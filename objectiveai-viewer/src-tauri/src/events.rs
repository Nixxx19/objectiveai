//! Event bus types shared by axum handlers, the mpsc channel that
//! buffers events while the React frontend boots, and Tauri's
//! `Emitter` that fans events out to the renderer.
//!
//! Every event is converted to [`EmittedEvent`] at the emit boundary
//! via [`Event::to_emitted`]. The envelope's `destination` field
//! doubles as the Tauri channel name — `"objectiveai"` for built-in
//! events, the plugin's repository name for plugin events. Plugin
//! repositories named `objectiveai` are refused at install time
//! (see `filesystem::plugins::InstallError::ReservedRepositoryName`),
//! so the channel namespaces can't collide.

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
    /// A plugin's viewer route was hit. `plugin` is the repository
    /// name (used to derive the Tauri channel `plugin-<repo>` + the
    /// emitted envelope's `destination`). `type` is the manifest-
    /// declared route type tag. `value` is the request body (JSON,
    /// or `Value::Null` for body-less requests).
    Plugin {
        plugin: String,
        r#type: String,
        value: serde_json::Value,
    },
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

impl Event {
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
            Event::Plugin { plugin, r#type, value } => EmittedEvent {
                destination: plugin.clone(),
                r#type: r#type.clone(),
                value: value.clone(),
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
    fn to_emitted_for_plugin_passes_through_type_and_value() {
        let e = Event::Plugin {
            plugin: "psyops".to_string(),
            r#type: "say".to_string(),
            value: json!({"to":"world"}),
        };
        let em = e.to_emitted();
        assert_eq!(em.destination, "psyops");
        assert_eq!(em.r#type, "say");
        assert_eq!(em.value, json!({"to":"world"}));
    }
}
