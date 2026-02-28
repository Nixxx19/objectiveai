//! Tool choice configuration for completions.

use crate::vector;
use serde::{Deserialize, Serialize};

/// Controls how the model uses tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolChoice {
    /// The model will not call any tools.
    #[serde(rename = "none")]
    None,
    /// The model decides whether to call tools.
    #[serde(rename = "auto")]
    Auto,
    /// The model must call at least one tool.
    #[serde(rename = "required")]
    Required,
    /// The model must call a specific function.
    #[serde(untagged)]
    Function(ToolChoiceFunction),
}

/// Specifies a specific function the model must call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoiceFunction {
    /// A specific function to call.
    Function {
        function: ToolChoiceFunctionFunction,
    },
}

/// The function name for a forced tool choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunctionFunction {
    /// The name of the function to call.
    pub name: String,
}

/// Creates tool choice configuration for vector completion voting.
///
/// When the output mode is `ToolCall`, returns the tool choice that forces
/// the LLM to call the response selection tool.
/// When there are request tools but output mode is not `ToolCall`, returns
/// `None` to prevent tool calls from interfering with voting.
pub fn new_for_vector(
    agent_output_mode: crate::agent::OutputMode,
    request_tools: Option<&[super::Tool]>,
) -> Option<ToolChoice> {
    if let crate::agent::OutputMode::ToolCall = agent_output_mode {
        Some(vector::completions::ResponseKey::tool_choice())
    } else if request_tools.is_some_and(|request_tools| !request_tools.is_empty()) {
        Some(ToolChoice::None)
    } else {
        Option::None
    }
}
