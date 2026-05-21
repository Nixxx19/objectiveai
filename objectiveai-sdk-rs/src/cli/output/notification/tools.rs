use crate::filesystem::tools::ManifestWithNameAndSource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Wire shape: `{"type":"notification","value":{"tools":[...]}}`.
/// Emitted by `objectiveai tools list`. One entry per `.json`
/// manifest discovered in `<base_dir>/tools/`; failures during
/// discovery are silently dropped (see `Client::list_tools`).
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[schemars(rename = "cli.output.notification.Tools")]
pub struct Tools {
    pub tools: Vec<ManifestWithNameAndSource>,
}

/// Wire shape: `{"type":"notification","value":{"tool": <manifest> | null}}`.
/// Emitted by `objectiveai tools get <name>`. The value is the
/// resolved `ManifestWithNameAndSource` when the manifest file exists
/// and parses, or JSON `null` when it doesn't (same silent-skip policy
/// as `list`).
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[schemars(rename = "cli.output.notification.Tool")]
pub struct Tool {
    pub tool: Option<ManifestWithNameAndSource>,
}
