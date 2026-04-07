//! Response types from the OpenRouter API.
//!
//! These types represent the upstream response format from OpenRouter and
//! provide methods to transform them into the downstream ObjectiveAI format.

mod chat_completion_chunk;
mod choice;
mod delta;
mod image;
mod object;
mod role;
mod usage;

pub use chat_completion_chunk::*;
pub use choice::*;
pub use delta::*;
pub use image::*;
pub use object::*;
pub use role::*;
pub use usage::*;

#[cfg(test)]
mod chat_completion_chunk_tests;