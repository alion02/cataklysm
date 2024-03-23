#![no_std]
#![allow(
	clippy::precedence, // Personal opinion
	clippy::comparison_chain, // Required for optimal performance at the time of writing
    clippy::absurd_extreme_comparisons, // Misfires for branches involving constants
)]
#![feature(portable_simd)]

extern crate alloc;

mod game;
mod influence;
mod params;
mod tt;
mod util;

use alloc::sync::Arc;
use core::{
    mem::transmute,
    ops::ControlFlow::{self, *},
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Neg, Not, Shl, ShlAssign,
        Shr, ShrAssign,
    },
    simd::{prelude::*, LaneCount, SupportedLaneCount},
    sync::atomic::AtomicBool,
};

use common::*;
use influence::*;
use params::*;
use tt::*;
use util::*;

const HAND: u32 = SIZE as u32;

const PADDING: usize = ROW_LEN - SIZE;
const ARR_LEN: usize = SIZE * ROW_LEN - PADDING;

const ROW: Bb = (1 << SIZE) - 1;
const COL: Bb = {
    let mut col: Bb = 1;
    while col.count_ones() < SIZE as u32 {
        col |= col << ROW_LEN;
    }
    col
};

const EDGE_BOTTOM: Bb = ROW;
const EDGE_TOP: Bb = ROW << (SIZE - 1) * ROW_LEN;
const EDGE_LEFT: Bb = COL;
const EDGE_RIGHT: Bb = COL << SIZE - 1;

const BOARD: Bb = ROW * COL;

#[repr(C)]
pub struct State {
    influence: Influence,
    owner: Bb,
    road: Bb,
    noble: Bb,
    tall: Bb,
    stones_left: Pair<u8>,
    caps_left: Pair<u8>,
    ply: u16,
    last_irreversible: u16,
    tt: *mut Bucket,
    tt_idx_mask: usize,
    stacks: [Stack; ROW_LEN],
    abort: Arc<AtomicBool>,
    inactive_abort: Arc<AtomicBool>,
    nodes: u64,
    generation: u32,
    half_komi: i32,
    hashes: WrappingArray<u64, 64>,
    killers: WrappingArray<u16, 32>,
}
