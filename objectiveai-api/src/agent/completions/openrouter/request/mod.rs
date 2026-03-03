//! Request types for OpenRouter API calls.
//!
//! This module transforms ObjectiveAI request types into the format expected
//! by the OpenRouter API, applying Agent configurations.

mod chat_completion_create_params;
/// Prompt construction for agent and vector completions.
pub mod prompt;
mod provider;
/// Response format construction for vector completions.
pub mod response_format;
mod prediction;
mod stream_options;
mod tool;
/// Tool choice construction for vector completions.
pub mod tool_choice;
/// Tools construction for vector completions.
pub mod tools;
mod usage;

pub use chat_completion_create_params::*;
pub use prediction::*;
pub use provider::*;
pub use stream_options::*;
pub use tool::*;
pub use usage::*;
