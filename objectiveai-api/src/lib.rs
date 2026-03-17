//! ObjectiveAI API server library.
//!
//! This crate provides the core implementation for the ObjectiveAI REST API,
//! which enables scoring, ranking, and simulating preferences using swarms of LLMs.
//!
//! # Modules
//!
//! - [`auth`] - Authentication and API key management
//! - [`chat`] - Chat completions with Swarm LLMs
//! - [`ctx`] - Request context and extensions
//! - [`swarm`] - Swarm management and retrieval
//! - [`agent`] - Agent management and retrieval
//! - [`error`] - Error response handling
//! - [`functions`] - Function execution and profile management
//! - [`util`] - Utility types for streaming and indexing
//! - [`vector`] - Vector completions for scoring and ranking

/// Authentication and API key management.
pub mod auth;
// /// Chat completions with Swarm LLMs.
// pub mod chat;
/// Request context and extensions for dependency injection.
pub mod ctx;
/// Swarm management, fetching, and retrieval.
pub mod swarm;
/// Agent management, fetching, and retrieval.
pub mod agent;
/// Error response handling and conversion.
pub mod error;
/// Local filesystem client for reading from git repositories.
pub mod filesystem;
/// Function execution, profile management, and computations.
pub mod functions;
/// GitHub API client for fetching functions and profiles.
pub mod github;
/// MCP (Model Context Protocol) types.
pub mod mcp;
/// ObjectiveAI HTTP client wrapper with per-request authorization.
pub mod objectiveai_http;
/// Utility types for streaming and choice indexing.
pub mod util;
/// Vector completions for scoring and ranking responses.
pub mod vector;

#[cfg(test)]
pub(crate) mod stream_harness;
