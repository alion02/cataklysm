#![allow(
	clippy::precedence, // Personal opinion
	clippy::comparison_chain, // Required for optimal performance at the time of writing
)]

mod action;
mod lut;
mod size;
mod square;
mod state;
mod tt;

extern crate alloc;

pub use state::State;

use crate::{action::*, lut::*, size::*, square::*, tt::*};

use common::{
    game::*,
    hash::*,
    pair::*,
    stack::*,
    state::{Direction::*, Piece::*, *},
    util::*,
};

use alloc::sync::Arc;
use core::{
    any::Any,
    fmt,
    mem::transmute,
    ops::{
        ControlFlow::{self, *},
        Index, IndexMut,
    },
    sync::atomic::{AtomicBool, Ordering::Relaxed},
};

const HAND: u32 = SIZE as u32;

const PADDING: usize = ROW_LEN - SIZE;
const ARR_LEN: usize = SIZE * ROW_LEN - PADDING;

const ROW: Bitboard = (1 << SIZE) - 1;
const COL: Bitboard = {
    let mut col: Bitboard = 1;
    while col.count_ones() < SIZE as u32 {
        col |= col << ROW_LEN;
    }
    col
};

const BOARD: Bitboard = ROW * COL;
