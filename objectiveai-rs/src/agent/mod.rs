//! Agent definitions, configuration, and completion API types.
//!
//! An **Agent** is a fully-specified configuration of a single upstream
//! language model. It encapsulates:
//!
//! - Model identity (which LLM to use)
//! - Prompt structure (prefix/suffix messages)
//! - Decoding parameters (temperature, top_p, etc.)
//! - Provider preferences and routing
//! - Output mode, reasoning settings, and verbosity
//!
//! # Content-Addressed Identity
//!
//! Agents use **content-addressed identifiers** - their ID is derived
//! deterministically from their full definition using XXHash3-128. This ensures:
//!
//! - Two identical definitions always produce the same ID
//! - IDs can be computed anywhere (server, client, browser via WASM)
//! - No hidden mutation or "latest version" ambiguity
//!
//! # Normalization
//!
//! Before computing an ID, definitions are normalized via [`AgentBase::prepare`]:
//!
//! - Default values are removed (e.g., `temperature: 1.0` becomes `None`)
//! - Empty collections are removed
//! - Collections are sorted for deterministic ordering

mod agent;
pub mod completions;
mod output_mode;
mod provider;
mod reasoning;
pub mod response;
mod stop;
mod verbosity;

pub use agent::*;
pub use output_mode::*;
pub use provider::*;
pub use reasoning::*;
pub use stop::*;
pub use verbosity::*;

#[cfg(feature = "http")]
mod http;

#[cfg(feature = "http")]
pub use http::*;
