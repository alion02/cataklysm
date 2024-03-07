#![no_std]
#![allow(
    clippy::precedence, // Personal opinion
)]

extern crate alloc;

pub mod color;
pub mod direction;
pub mod pair;
pub mod piece;

pub use color::{Color::*, *};
pub use direction::{Direction::*, *};
pub use pair::*;
pub use piece::{Piece::*, *};
