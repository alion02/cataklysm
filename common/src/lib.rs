#![no_std]
#![allow(
    clippy::precedence, // Personal opinion
)]

extern crate alloc;

pub mod color;
pub mod direction;
pub mod event_log;
#[macro_use]
pub mod opt_fence;
pub mod out;
pub mod pair;
pub mod piece;
pub mod wrapping_array;

pub use color::{Color::*, *};
pub use direction::{Direction::*, *};
pub use event_log::*;
pub use out::*;
pub use pair::*;
pub use piece::{Piece::*, *};
pub use wrapping_array::*;
pub use EventKind::*;
