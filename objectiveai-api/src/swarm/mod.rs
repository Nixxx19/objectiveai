//! Swarm management, fetching, and retrieval.
//!
//! Swarms are collections of Swarm LLMs used together for voting.
//! This module provides clients for listing, retrieving, and fetching swarms.

mod client;
/// Fetchers for retrieving swarm definitions by ID.
pub mod fetcher;
/// Retrieval clients for listing swarms and getting usage statistics.
pub mod retrieval_client;

pub use client::*;
