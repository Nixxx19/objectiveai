//! ObjectiveAI SDK for Rust.
//!
//! This crate provides data structures, validation, and client-side compilation
//! for the ObjectiveAI API - a platform for scoring, ranking, and simulating
//! preferences using ensembles of agents.
//!
//! # Core Concepts
//!
//! - **Agent**: A configured instance of a single upstream language model
//! - **Ensemble**: A collection of Agents used together for voting
//! - **Vector Completion**: Runs multiple agents to vote on responses, producing weighted scores
//! - **Function**: A composable scoring pipeline built from Vector Completions
//! - **Profile**: Learned weights for a Function, trained on example data
//!
//! # Features
//!
//! - `http` (default): Enables the HTTP client for making API requests
//!
//! # Modules
//!
//! - [`auth`] - API authentication types
//! - [`agent`] - Agent definitions, configuration, and completion APIs
//! - [`ensemble`] - Ensemble definitions and validation
//! - [`error`] - Error types
//! - [`functions`] - Function definitions, execution, and client-side compilation
//! - [`prefixed_uuid`] - UUID utilities
//! - [`vector`] - Vector completion APIs
//!
//! When the `http` feature is enabled:
//! - [`HttpClient`] - HTTP client for API requests
//! - [`HttpError`] - HTTP error types

pub mod auth;
pub mod agent;
pub mod ensemble;
pub mod error;
pub mod functions;
pub mod prefixed_uuid;
pub mod upstream;
pub mod vector;

#[cfg(test)]
mod util;

#[cfg(feature = "http")]
mod http;

#[cfg(feature = "http")]
pub use http::*;
