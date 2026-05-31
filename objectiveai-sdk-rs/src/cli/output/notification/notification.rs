use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::NotificationValue;

/// One emitted notification. The payload's keys flatten directly
/// into the top-level JSON object via `#[serde(flatten)]` — there
/// is no `value` wrapper or `type:"notification"` envelope.
///
/// `Notification` has no struct-level `agent_id` field. Inner
/// payloads that name an agent (e.g. `Spawned`, `MessageQueued`,
/// `Inactive`, `Me`) carry agent_id themselves — flattening would
/// otherwise collide on the wire. For payloads with no inherent
/// agent_id (`Ok`, `LogContent`, `Mcp`, …), the cli session's
/// agent_id is stamped at JSON-serialize time by
/// [`super::super::Handle::emit`] iff the resulting JSON object
/// doesn't already contain one.
///
/// Wire:
///   `{"type":"<typed-variant>",…fields…}` for typed variants.
///   `{…object-keys…}` for `Other` payloads.
///   Handle additionally injects `"agent_id":"<cli-session-id>"`
///   at JSON level when no inner agent_id is present.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[schemars(rename = "cli.output.notification.Notification")]
pub struct Notification {
    #[serde(flatten)]
    pub value: NotificationValue,
}
