use crate::vector;

pub fn new_for_vector(
    ensemble_llm_output_mode: objectiveai::ensemble_llm::OutputMode,
    request_tools: Option<&[objectiveai::chat::completions::request::Tool]>,
) -> Option<objectiveai::chat::completions::request::ToolChoice> {
    if let objectiveai::ensemble_llm::OutputMode::ToolCall = ensemble_llm_output_mode {
        Some(vector::completions::ResponseKey::tool_choice())
    } else if request_tools.is_some_and(|request_tools| !request_tools.is_empty()) {
        Some(objectiveai::chat::completions::request::ToolChoice::None)
    } else {
        None
    }
}
