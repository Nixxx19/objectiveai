//! Agent management, fetching, and retrieval.
//!
//! An Agent is a fully-specified configuration of a single upstream LLM,
//! including model identity, prompt structure, decoding parameters, and output mode.

mod client;
pub mod completions;
/// Fetchers for retrieving Agent definitions by ID.
pub mod fetcher;
/// Retrieval clients for listing Agents and getting usage statistics.
pub mod retrieval_client;

pub use client::*;
