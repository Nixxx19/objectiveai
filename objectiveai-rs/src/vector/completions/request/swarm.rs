//! Swarm specification for vector completion requests.

use crate::swarm;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Specifies which Swarm to use for a vector completion.
///
/// Swarms can be referenced by ID or provided inline. The untagged
/// deserialization allows either a string ID or a full [`SwarmBase`]
/// definition in JSON.
///
/// # Examples
///
/// By ID:
/// ```json
/// "swarm": "ens_abc123"
/// ```
///
/// Inline definition:
/// ```json
/// "swarm": {
///   "llms": [
///     {"model": "openai/gpt-4o", "output_mode": "json_schema", "count": 2},
///     {"model": "google/gemini-3.0-pro", "output_mode": "tool_call"}
///   ]
/// }
/// ```
///
/// [`SwarmBase`]: crate::swarm::SwarmBase
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[serde(untagged)]
#[schemars(rename = "vector.completions.request.Swarm")]
pub enum Swarm {
    /// Reference an existing Swarm by its ID.
    Id(String),
    /// Provide an inline Swarm definition.
    Provided(swarm::SwarmBase),
}
