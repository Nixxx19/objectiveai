//! Error response handling, conversion to Axum responses, and error endpoint client.

mod client;
mod response_error_ext;

pub use client::*;
pub use response_error_ext::*;
