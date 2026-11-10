//! The terminal: printing results, asking questions and reviewing hunks.

mod output;
mod prompt;
mod review;

pub use output::Output;
pub use prompt::{ask, confirm};
pub use review::review;
