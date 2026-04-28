use serde::{Deserialize, Serialize};

/// Per-turn options. Mirrors `TurnOptions` in the Python SDK.
///
/// `output_schema` is a JSON Schema document — by definition arbitrary JSON
/// from our perspective. The Python SDK writes it to a tempfile and passes
/// `--output-schema <path>` to the codex binary; the equivalent flow lives
/// in [`super::super::CodexExecArgs::to_command_args`].
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TurnOptions {
    #[serde(
        rename = "output_schema",
        alias = "outputSchema",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub output_schema: Option<serde_json::Value>,
}
