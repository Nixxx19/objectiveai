//! Fetcher for Function definitions from remote sources.

mod fetcher;
pub mod filesystem;
pub mod github;
pub mod mock;
mod response;
mod router;

pub use fetcher::*;
pub use response::*;
pub use router::*;
