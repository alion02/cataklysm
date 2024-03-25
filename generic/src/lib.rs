#![no_std] // Ensures stuff like WASM will just work
#![allow(
	clippy::precedence, // Personal opinion
	clippy::comparison_chain, // Required for optimal performance at the time of writing
    clippy::absurd_extreme_comparisons, // Misfires for branches involving constants
)]
#![feature(
    portable_simd, // Used extensively for performance
    strict_provenance, // Provides `with_addr`
)]

extern crate alloc;

mod influence;
mod params;
mod rules;
mod tt;
mod util;

use alloc::sync::Arc;
use core::{
    mem::transmute,
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign,
        ControlFlow::{self, *},
        Neg, Not, Shl, ShlAssign, Shr, ShrAssign,
    },
    simd::{prelude::*, LaneCount, SupportedLaneCount},
    sync::atomic::{AtomicBool, AtomicU64, Ordering::*},
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

const STACK_CAP: usize = Stack::BITS as usize - 1;

const HASH_SIDE_TO_MOVE: u64 = 0xf812ec2e34a9c388u64; // TODO: alt-seed

#[allow(clippy::declare_interior_mutable_const)]
const ATOMIC_U64_ZERO: AtomicU64 = AtomicU64::new(0);
static HASH_PC_SQ: [AtomicU64; 256] = [ATOMIC_U64_ZERO; 256];
static HASH_STACK: [AtomicU64; ((2 << HAND) - 2) * STACK_CAP * ARR_LEN] =
    [ATOMIC_U64_ZERO; ((2 << HAND) - 2) * STACK_CAP * ARR_LEN];

pub struct State<'a> {
    update: &'a mut UpdateState,
    copy: &'a mut CopyState,
}

#[repr(C)]
struct UpdateState {
    influence: Influence,
    tt: *mut Bucket,
    tt_idx_mask: usize,
    abort: Arc<AtomicBool>,
    nodes: u64,
    stones_left: Pair<u8>,
    caps_left: Pair<u8>,
    generation: u8,
    half_komi: i8,
    log: EventLog,
    stacks: [Stack; ROW_LEN],
    inactive_abort: Arc<AtomicBool>,
    hashes: WrappingArray<u64, 64>,
    killers: WrappingArray<u16, 32>,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CopyState {
    owner: Bb,
    road: Bb,
    noble: Bb,
    tall: Bb,
    ply: u16,
    last_irreversible: u16,
}

#[derive(Clone, Copy)]
struct Unmake {
    influence: Simd<Bb, 4>,
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
    ($s:ident, $c:expr, $e:expr) => {
        if $s.update.log.should_log() {
            if $c {
                $s.update.log.log(Event {
                    ply: $s.copy.ply,
                    kind: $e,
                })
            }
        }
    };

    ($s:ident, $e:expr) => {
        log!($s, true, $e)
    };
}

impl<'a> State<'a> {
    #[inline]
    fn lend(&mut self) -> State {
        State {
            update: self.update,
            copy: self.copy,
        }
    }

    #[inline]
    fn player(&self) -> bool {
        self.copy.ply & 1 != 0
    }

    #[inline]
    fn piece(&self) -> Bb {
        self.copy.road | self.copy.noble
    }
}
