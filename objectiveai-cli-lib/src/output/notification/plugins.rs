use objectiveai::filesystem::plugins::ManifestWithNameAndSource;
use serde::{Deserialize, Serialize};

/// Wire shape: `{"type":"notification","value":{"plugins":[...]}}`.
/// Emitted by `objectiveai plugins list`. One entry per `.json`
/// manifest discovered in `<base_dir>/plugins/`; failures during
/// discovery are silently dropped (see `Client::list_plugins`).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Plugins {
    pub plugins: Vec<ManifestWithNameAndSource>,
}
