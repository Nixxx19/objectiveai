//! Plugin protocol wire types and (future) helpers.

mod output;
mod response;

pub use output::*;
pub use response::*;

#[cfg(test)]
mod output_tests;
