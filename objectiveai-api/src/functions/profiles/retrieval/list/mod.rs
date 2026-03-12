//! Client for listing Profiles from multiple sources.

mod client;
/// Filesystem list implementation.
pub mod filesystem;
/// Mock list implementation.
pub mod mock;
/// ObjectiveAI API list implementation.
pub mod objectiveai;
mod router;

pub use client::*;
pub use router::*;
