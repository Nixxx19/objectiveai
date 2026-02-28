//! Vector completions request and response types.

pub mod cache;
pub mod request;
pub mod response;
mod response_key;
pub mod vector_responses;

pub use response_key::*;

#[cfg(feature = "http")]
mod http;

#[cfg(feature = "http")]
pub use http::*;
