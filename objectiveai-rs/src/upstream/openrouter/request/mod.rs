//! Request types for OpenRouter API calls.
//!
//! This module transforms ObjectiveAI request types into the format expected
//! by the OpenRouter API, applying Agent configurations.

mod agent_completion_create_params;
/// Prompt construction for agent and vector completions.
pub mod prompt;
mod provider;
/// Response format construction for vector completions.
pub mod response_format;
mod stream_options;
/// Tool choice construction for vector completions.
pub mod tool_choice;
/// Tools construction for vector completions.
pub mod tools;
mod usage;

pub use agent_completion_create_params::*;
pub use provider::*;
pub use stream_options::*;
pub use usage::*;
