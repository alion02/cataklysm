#![allow(
	clippy::precedence, // Personal opinion
	clippy::comparison_chain, // Required for optimal performance at the time of writing
)]

pub mod game;
pub mod pair;
pub mod stack;
pub mod state;
pub mod util;

// Do not include the template module outside of a test build
// Workaround for rust-analyzer failing to lint warnings if a cfg(feature = ...) is used instead
#[cfg(test)]
mod template;
