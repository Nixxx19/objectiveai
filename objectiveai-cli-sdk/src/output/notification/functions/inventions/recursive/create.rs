use serde::{Deserialize, Serialize};

/// Result of `functions inventions recursive create`.
///
/// Wire: `{"type":"notification","inventions":[...InventionResultItem...]}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Inventions {
    pub inventions: Vec<InventionResultItem>,
}

/// One entry per invention produced. `name` is the human-readable
/// state label (e.g. `"alpha_scalar_leaf"`); `path` is set only when
/// the invention successfully resolved to a remote definition
/// (`None` if it failed). Per-invention failures surface live via
/// `Output::Error` notifications during streaming, not on this struct.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventionResultItem {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<objectiveai_sdk::RemotePath>,
}
