mod client;
mod error;
pub mod recursive;
pub mod usage_handler;

pub use client::*;
pub use error::*;

pub(crate) use client::{
    extract_description, publish_filesystem, publish_github,
};

#[cfg(test)]
mod client_tests;
