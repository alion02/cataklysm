#![no_std]
#![allow(
    clippy::precedence, // Personal opinion
)]

extern crate alloc;

pub mod color;
pub mod direction;
pub mod hash;
pub mod pair;
pub mod piece;
pub mod wrapping_array;

pub use color::{Color::*, *};
pub use direction::{Direction::*, *};
pub use hash::*;
pub use pair::*;
pub use piece::{Piece::*, *};
pub use wrapping_array::*;
