//! Response types for agent completions.
//!
//! This module contains types for parsing agent completion responses:
//!
//! - [`unary`] - Non-streaming (single response) types
//! - [`streaming`] - Streaming (Server-Sent Events) types
//! - Common types: [`FinishReason`], [`Usage`], [`Role`], [`Logprobs`]

mod assistant_response;
mod finish_reason;
mod logprobs;
pub mod streaming;
mod tool_response;
pub mod unary;
mod usage;
pub mod util;

pub use assistant_response::*;
pub use finish_reason::*;
pub use logprobs::*;
pub use tool_response::*;
pub use usage::*;
