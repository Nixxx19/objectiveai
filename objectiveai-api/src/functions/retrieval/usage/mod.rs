//! Client for retrieving Function usage statistics.

mod client;
/// ObjectiveAI API usage implementation.
pub mod objectiveai;
mod router;

pub use client::*;
pub use router::*;
