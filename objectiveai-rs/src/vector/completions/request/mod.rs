//! Request types for vector completions.
//!
//! - [`VectorCompletionCreateParams`] - The main request structure

mod profile;
mod vector_completion_create_params;

pub use profile::*;
pub use vector_completion_create_params::*;
