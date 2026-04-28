use serde::{Deserialize, Serialize};

/// Carries no payload — the discriminator on the parent enum is the only
/// signal. Modeled as a unit struct so the wire shape stays `{"type":"turn.started"}`.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnStartedEvent {}
