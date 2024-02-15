#![allow(
	clippy::precedence, // Personal opinion
	clippy::comparison_chain, // Required for optimal performance at the time of writing
)]

mod action;
mod influence;
mod lut;
mod params;
mod pv;
mod square;
mod state;
mod tt;
mod util;

extern crate alloc;

pub use state::State;

use crate::{action::*, influence::*, lut::*, params::*, pv::*, square::*, tt::*, util::*};

use common::{
    game::*,
    hash::*,
    pair::*,
    params::*,
    stack::*,
    state::{Direction::*, Piece::*, *},
    util::*,
};

use alloc::sync::Arc;
use core::{
    any::Any,
    array::from_fn as make_arr,
    cell::RefCell,
    cmp::min,
    fmt,
    mem::transmute,
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign,
        ControlFlow::{self, *},
        Deref, Index, IndexMut,
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

const EDGE_BOTTOM: Bitboard = ROW;
const EDGE_TOP: Bitboard = ROW << (SIZE - 1) * ROW_LEN;
const EDGE_LEFT: Bitboard = COL;
const EDGE_RIGHT: Bitboard = COL << SIZE - 1;

const BOARD: Bitboard = ROW * COL;
