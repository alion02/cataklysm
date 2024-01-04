#![allow(
	clippy::precedence, // Personal opinion
	clippy::comparison_chain, // Required for optimal performance at the time of writing
)]

pub mod game;
pub mod pair;
pub mod size;
pub mod stack;
pub mod state;
pub mod util;

// Hide the template module behind a feature indicated as private
#[cfg(feature = "__template")]
mod template;
