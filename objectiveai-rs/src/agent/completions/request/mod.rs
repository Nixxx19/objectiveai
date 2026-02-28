//! Request types for agent completions.
//!
//! - [`AgentCompletionCreateParams`] - The main request structure
//! - [`Message`] - Agent messages (system, user, assistant, tool, developer)
//! - [`Model`] - Either an inline Ensemble LLM or the ID of a previously used one
//! - [`Tool`] - Tool/function definitions for function calling
//! - [`ToolChoice`] - How the model should use tools
//! - [`ResponseFormat`] - Output format constraints (text, JSON, JSON schema)
//! - [`Prediction`] - Predicted output for speculative decoding
//! - [`Provider`] - Provider routing preferences

mod agent_completion_create_params;
mod message;
mod model;
mod prediction;
mod provider;
mod response_format;
mod tool;
mod tool_choice;

pub use agent_completion_create_params::*;
pub use message::*;
pub use model::*;
pub use prediction::*;
pub use provider::*;
pub use response_format::*;
pub use tool::*;
pub use tool_choice::*;
