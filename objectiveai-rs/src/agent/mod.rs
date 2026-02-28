//! Agent completion API types.
//!
//! This module provides types for the ObjectiveAI agent completions API.
//! While inspired by the OpenAI agent completions format, it diverges in
//! several ways - notably, the `model` field must be an Ensemble LLM
//! configuration rather than a simple model string.

pub mod completions;
