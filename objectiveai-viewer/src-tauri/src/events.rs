//! Event bus. Built-in axum routes and dynamic plugin routes both
//! fan into the same struct; serve() emits it as-is under the
//! `destination` Tauri channel name.
//!
//! Channel-name namespacing: `"objectiveai"` is reserved as the
//! built-in destination; plugin repositories named "objectiveai"
//! are refused at install time (see
//! `filesystem::plugins::InstallError::ReservedRepositoryName`), so
//! a plugin can't shadow built-in events.

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Unified shape for every event the viewer emits. `destination`
/// is `"objectiveai"` for built-in events or the plugin's repository
/// name for plugin events. `type` is a snake_case discriminator
/// (built-ins: `agent_completions` / `functions_executions` /
/// `functions_inventions_recursive` / `laboratories_executions`;
/// plugins: whatever they declared in their manifest's
/// `viewer_routes[i].type`). `value` is the raw JSON body — no
/// typed coercion happens server-side.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub destination: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub value: serde_json::Value,
}

pub type EventReceiver = mpsc::UnboundedReceiver<Event>;
pub type EventSender = mpsc::UnboundedSender<Event>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn event_serializes_to_destination_type_value_envelope() {
        let e = Event {
            destination: "objectiveai".to_string(),
            r#type: "agent_completions".to_string(),
            value: json!({"id": "abc"}),
        };
        let s = serde_json::to_string(&e).unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["destination"], "objectiveai");
        assert_eq!(v["type"], "agent_completions");
        assert_eq!(v["value"], json!({"id": "abc"}));

        // Round-trip back into Event.
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(back.destination, "objectiveai");
        assert_eq!(back.r#type, "agent_completions");
        assert_eq!(back.value, json!({"id": "abc"}));
    }
}
