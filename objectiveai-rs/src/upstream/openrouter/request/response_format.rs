//! Response format construction for vector completions.

use crate::vector;

/// The format of the model's response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    /// Plain text response (default).
    Text,
    /// Response must be valid JSON.
    JsonObject,
    /// Response must conform to a JSON schema.
    JsonSchema { json_schema: JsonSchema },
    /// Response must conform to a grammar.
    Grammar { grammar: String },
    /// Response must be valid Python code.
    Python,
}

/// A JSON schema for structured output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchema {
    /// The name of the schema.
    pub name: String,
    /// A description of the schema's purpose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The JSON Schema definition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    /// Whether to enforce strict schema validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// Creates a response format for vector completion voting.
///
/// When the output mode is `JsonSchema`, returns a JSON schema that constrains
/// the LLM's output to select one of the available response keys.
/// Returns None for other output modes.
pub fn new_for_vector(
    vector_pfx_indices: &[(String, usize)],
    agent_output_mode: crate::agent::OutputMode,
    agent_synthetic_reasoning: Option<bool>,
) -> Option<crate::agent::completions::request::ResponseFormat> {
    if let crate::agent::OutputMode::JsonSchema = agent_output_mode {
        Some(vector::completions::ResponseKey::response_format(
            vector_pfx_indices
                .iter()
                .map(|(key, _)| key.clone())
                .collect(),
            agent_synthetic_reasoning.unwrap_or(false),
        ))
    } else {
        None
    }
}
