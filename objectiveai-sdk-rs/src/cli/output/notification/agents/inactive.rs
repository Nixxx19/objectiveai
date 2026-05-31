//! `Inactive` — emitted by `agents read subscribe` when no
//! cli-stream is currently writing for the requested spawned agent
//! AND no unread messages remain. The combination is "there's nothing
//! to wait for, and nothing to drain" — the caller should treat the
//! subscribe as terminated without a row event.
//!
//! See [`super::AgentItems`] for the wire shape used when subscribe
//! finds unread rows or a `Row` event lands; `Inactive` only fires
//! on the strictly empty + dormant case.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The spawned agent identified by `agent_id` has no active
/// cli-stream writer AND no unread queue rows from this caller's
/// perspective. `agent_id` carries the sub-id form the caller passed
/// (the lineage prefix is stripped), matching how `AgentItems` is
/// emitted.
///
/// Wire: `{"type":"notification","value":{"kind":"inactive","agent_id":"<sub>"}}`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[schemars(rename = "cli.output.notification.agents.Inactive")]
pub struct Inactive {
    pub agent_id: String,
}
