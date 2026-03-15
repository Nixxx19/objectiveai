//! Fetcher for Profile definitions from remote sources.

mod fetcher;
pub mod filesystem;
pub mod github;
pub mod mock;
mod router;

pub use fetcher::*;
pub use router::*;
