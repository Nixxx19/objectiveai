//! Client for listing Function-Profile pairs from multiple sources.

mod client;
/// ObjectiveAI API list implementation.
pub mod objectiveai;
mod router;

pub use client::*;
pub use router::*;
