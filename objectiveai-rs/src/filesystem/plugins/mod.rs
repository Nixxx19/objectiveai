//! Plugin discovery and manifest types.

mod client;
#[cfg(feature = "http")]
mod install_error;
mod manifest;
mod platform;

pub use client::*;
#[cfg(feature = "http")]
pub use install_error::*;
pub use manifest::*;
pub use platform::*;

#[cfg(test)]
mod client_tests;
#[cfg(all(test, feature = "http"))]
mod install_error_tests;
#[cfg(all(test, feature = "http"))]
mod install_tests;
#[cfg(test)]
mod manifest_tests;
#[cfg(test)]
mod platform_tests;
