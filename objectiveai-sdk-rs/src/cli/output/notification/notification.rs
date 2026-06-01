use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::NotificationValue;

/// One emitted notification. The payload's keys flatten directly
/// into the top-level JSON object via `#[serde(flatten)]` — there
/// is no `value` wrapper or `type:"notification"` envelope.
///
/// `Notification` has no struct-level `agent_instance_hierarchy` field. Inner
/// payloads that name an agent (e.g. `Spawned`, `MessageQueued`,
/// `Inactive`, `Me`) carry agent_instance_hierarchy themselves — flattening would
/// otherwise collide on the wire. For payloads with no inherent
/// agent_instance_hierarchy (`Ok`, `LogContent`, `Mcp`, …), the cli session's
/// agent_instance_hierarchy is stamped at JSON-serialize time by
/// [`super::super::Handle::emit`] iff the resulting JSON object
/// doesn't already contain one.
///
/// Wire:
///   `{"type":"<typed-variant>",…fields…}` for typed variants.
///   `{…object-keys…}` for `Other` payloads.
///   Handle additionally injects `"agent_instance_hierarchy":"<cli-session-id>"`
///   at JSON level when no inner agent_instance_hierarchy is present.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[schemars(rename = "cli.output.notification.Notification")]
pub struct Notification {
    #[serde(flatten)]
    pub value: NotificationValue,
}
