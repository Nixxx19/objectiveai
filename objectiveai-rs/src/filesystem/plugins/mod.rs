//! Plugin discovery and manifest types.

mod client;
mod manifest;
mod platform;

pub use client::*;
pub use manifest::*;
pub use platform::*;

#[cfg(test)]
mod client_tests;
#[cfg(test)]
mod manifest_tests;
#[cfg(test)]
mod platform_tests;
