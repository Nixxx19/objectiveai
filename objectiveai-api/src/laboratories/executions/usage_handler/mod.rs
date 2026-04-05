//! Usage handling for laboratory executions.
//!
//! Provides traits and implementations for recording usage after
//! laboratory execution completes.

mod log_usage_handler;
mod usage_handler;

pub use log_usage_handler::*;
pub use usage_handler::*;
