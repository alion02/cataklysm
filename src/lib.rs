#![allow(clippy::precedence, clippy::comparison_chain)]

pub mod game;
pub mod pair;
pub mod stack;
pub mod state;
pub mod util;

// Do not include the template module outside of a test build
// Workaround for rust-analyzer failing to lint warnings if a cfg(feature = ...) is used instead
#[cfg(test)]
mod template;
