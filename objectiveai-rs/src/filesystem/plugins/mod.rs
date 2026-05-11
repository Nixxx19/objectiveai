//! Plugin discovery and manifest types.

mod client;
mod manifest;

pub use client::*;
pub use manifest::*;

#[cfg(test)]
mod manifest_tests;
