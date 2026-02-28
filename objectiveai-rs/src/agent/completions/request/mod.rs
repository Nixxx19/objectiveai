//! Request types for agent completions.
//!
//! - [`AgentCompletionCreateParams`] - The main request structure
//! - [`Message`] - Agent messages (system, user, assistant, tool, developer)
//! - [`Model`] - Either an inline Agent or the ID of a previously used one
//! - [`ResponseFormat`] - Output format constraints (text, JSON, JSON schema)
//! - [`Provider`] - Provider routing preferences

mod agent_completion_create_params;
mod message;
mod model;
mod provider;
mod response_format;

pub use agent_completion_create_params::*;
pub use message::*;
pub use model::*;
pub use provider::*;
pub use response_format::*;
