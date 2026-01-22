use crate::vector;

pub fn new_for_vector(
    vector_pfx_indices: &[(String, usize)],
    ensemble_llm_output_mode: objectiveai::ensemble_llm::OutputMode,
    ensemble_llm_synthetic_reasoning: Option<bool>,
) -> Option<objectiveai::chat::completions::request::ResponseFormat> {
    if let objectiveai::ensemble_llm::OutputMode::JsonSchema = ensemble_llm_output_mode {
        Some(vector::completions::ResponseKey::response_format(
            vector_pfx_indices
                .iter()
                .map(|(key, _)| key.clone())
                .collect(),
            ensemble_llm_synthetic_reasoning.unwrap_or(false),
        ))
    } else {
        None
    }
}
