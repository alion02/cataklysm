#![no_std] // Ensures stuff like WASM will just work
#![allow(
	clippy::precedence, // Personal opinion
	clippy::comparison_chain, // Required for optimal performance at the time of writing
    clippy::absurd_extreme_comparisons, // Misfires for branches involving constants
    internal_features, // We use `core_intrinsics`
)]
#![feature(
    portable_simd, // Used extensively for performance
    strict_provenance, // Provides `with_addr`
    core_intrinsics, // Provides `unchecked_shl`, `unlikely`
)]

extern crate alloc;

mod influence;
mod params;
mod rules;
mod tables;
mod tt;
mod util;

use alloc::sync::Arc;
use core::{
    hint::black_box,
    intrinsics::{unchecked_shl, unlikely},
    mem::transmute,
    ops::ControlFlow::{self, *},
    simd::{prelude::*, LaneCount, SupportedLaneCount},
    sync::atomic::{AtomicBool, AtomicU8, Ordering::*},
};

use common::*;
use influence::*;
use params::*;
use tables::*;
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

const STACK_CAP: usize = Stack::BITS as usize - 1;

const TAG_OFFSET: u32 = (ARR_LEN - 1).ilog2() + 1;
const ROAD_TAG_OFFSET: u32 = TAG_OFFSET;
const NOBLE_TAG_OFFSET: u32 = TAG_OFFSET + 1;
const PAT_OFFSET: u32 = TAG_OFFSET + 2;

const FLAT_TAG: u16 = 1;
const WALL_TAG: u16 = 2;
const CAP_TAG: u16 = 3;

const HASH_SIDE_TO_MOVE: u64 = 0xf812ec2e34a9c388u64; // TODO: alt-seed

pub struct State<'a> {
    update: &'a mut UpdateState,
    copy: &'a mut CopyState,
}

#[repr(C)]
struct UpdateState {
    tt: *mut u8,
    tt_idx_mask: usize,
    abort: Arc<AtomicBool>,
    nodes: u64,
    stones_left: Pair<u8>,
    caps_left: Pair<u8>,
    generation: u8,
    half_komi: i8,
    log: EventLog,
    stacks: [Stack; ARR_LEN],
    inactive_abort: Arc<AtomicBool>,
    hashes: WrappingArray<u64, 64>,
    killers: WrappingArray<u16, 32>,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CopyState {
    /// Influence maps extending from each edge. Contains all contiguous road tiles and their
    /// neighbors, as well as all edge tiles. May contain garbage bits outside the bitboards.
    influence: Influence,
    /// Specifies all tiles belonging to the active player. No empty tiles or garbage bits. Empty
    /// on plies 0 and 1.
    own: Bb,
    road: Bb,
    noble: Bb,
    tall: Bb,
    ply: u16,
    last_irreversible: u16,
}

#[derive(Clone, Copy)]
struct Unmake {
    kind: UnmakeKind,
}

#[derive(Clone, Copy)]
union UnmakeKind {
    place: UnmakePlace,
    spread: UnmakeSpread,
}

#[derive(Clone, Copy)]
struct UnmakePlace {
    pieces_left_ptr: usize,
}

#[derive(Clone, Copy)]
struct UnmakeSpread {}

#[macro_export]
macro_rules! log {
    ($s:ident, $e:expr, $w:expr, $c:expr) => {
        if $s.update.log.try_log($w) {
            if $c {
                $s.update.log.log(Event {
                    ply: $s.copy.ply,
                    kind: $e,
                })
            }
        }
    };

    ($s:ident, $e:expr) => {
        log!($s, $e, 1, true)
    };
}

#[inline]
fn sq(action: u16) -> u16 {
    let r = action & (1 << TAG_OFFSET) - 1;
    debug_assert_ne!(BOARD & 1 << r, 0);
    r
}

#[inline]
fn pat(action: u16) -> u16 {
    action >> PAT_OFFSET
}

impl<'a> State<'a> {
    #[inline]
    #[allow(clippy::new_without_default)] // Will have parameters.
    pub fn new() -> Self {
        init();
        todo!()
    }

    #[inline]
    fn lend(&mut self) -> State {
        State {
            update: self.update,
            copy: self.copy,
        }
    }

    #[inline]
    fn player(&self) -> Color {
        (self.copy.ply as u32 & 1 != 0).into()
    }

    #[inline]
    fn is_first_move(&self) -> bool {
        self.copy.ply < 2
    }

    #[inline]
    fn piece(&self) -> Bb {
        self.copy.road | self.copy.noble
    }
}
