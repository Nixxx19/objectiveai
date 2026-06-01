use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Wire shape: `{"type":"notification","value":{"kind":"me","agent_instance_hierarchy":"..."}}`.
/// Emitted by `objectiveai agents me`. The configured self agent id,
/// read from `Config.agent_instance_hierarchy` — sourced from `OBJECTIVEAI_AGENT_INSTANCE_HIERARCHY`
/// for direct CLI, or from the `X-OBJECTIVEAI-AGENT-INSTANCE-HIERARCHY` header when
/// running under the MCP server (which defaults to `"MCP"` when the
/// header is absent).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[schemars(rename = "cli.output.notification.Me")]
pub struct Me {
    pub agent_instance_hierarchy: String,
}
