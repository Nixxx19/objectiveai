//! Response format construction for vector completions.

use crate::vector;

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
