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

mod game;
mod influence;
mod params;
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
use EventKind::*;

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
    stacks: [Stack; ROW_LEN],
    inactive_abort: Arc<AtomicBool>,
    hashes: WrappingArray<u64, 64>,
    killers: WrappingArray<u16, 32>,
    log: EventLog<Event>,
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

#[derive(Hash, PartialEq, Eq)]
struct Event {
    ply: u16,
    kind: EventKind,
}

#[derive(Hash, PartialEq, Eq)]
enum EventKind {
    MakePlaceFlat,
    MakePlaceWall,
    MakePlaceCap,
    PlacementExpansionIterations(u8),
}

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

    #[inline]
    fn for_actions<B, C>(
        &mut self,
        mut acc: C,
        f: fn(State, C, u16) -> ControlFlow<B, C>,
    ) -> ControlFlow<B, C> {
        let empty = self.piece() ^ BOARD;
        macro_rules! for_placements {
            ($pc:expr) => {
                let mut left = empty;
                // Assume actions are not generated on completed games.
                loop {
                    acc = f(self.lend(), acc, left.trailing_zeros() as u16 | $pc << 6)?;
                    left &= left - 1;
                    if left == 0 {
                        break;
                    }
                }
            };
        }

        // Only do stone placements if we have any stones in reserve. We're technically checking
        // the wrong reserves on the first move because we assume this passes anyway.
        if self.update.stones_left[self.player()] > 0 {
            for_placements!(1);

            // If it's the first move, skip over all the other kinds of actions. This check gets
            // skipped if we don't have stones, but we again don't care.
            if self.copy.ply < 2 {
                return Continue(acc);
            }

            // Wall placements.
            for_placements!(2);
        }

        if self.update.caps_left[self.player()] > 0 {
            // Cap placements.
            for_placements!(3);
        }

        Continue(acc)
    }

    #[no_mangle]
    #[inline]
    fn make(&mut self, new: &mut Out<CopyState>, unmake: &mut Out<Unmake>, action: u16) {
        let new_ply = self.copy.ply + 1;
        new.val.ply = new_ply;

        let mut hash = self.update.hashes[self.copy.ply] ^ HASH_SIDE_TO_MOVE;

        let player = self.player();
        let pat = action >> 8;
        let sq = action & 0x3f;
        if pat == 0 {
            hash ^= HASH_PC_SQ[action as usize].load(Relaxed);
            hash ^= HASH_STACK[0].load(Relaxed); // TODO

            log!(self, match action >> 6 {
                1 => MakePlaceFlat,
                2 => MakePlaceWall,
                3 => MakePlaceCap,
                _ => unreachable!(),
            });

            // Check if it's the first move (ply 0 or 1).
            let player = player ^ (self.copy.ply < 2); // Strange codegen. AND with 1 thrice.
            let inf = &mut self.update.influence.pair_mut()[player];

            // Unconditionally store. Not worth checking if it needs unmaking.
            unmake.val.influence = *inf;

            // Remove the appropriate piece from the reserves.
            let pieces_left_ref = &mut (if action >= 3 << 6 {
                &mut self.update.caps_left
            } else {
                &mut self.update.stones_left
            })[player];
            *pieces_left_ref -= 1;
            unmake.val.kind.place.pieces_left_ptr = (pieces_left_ref as *mut u8).addr();

            let road_bit = (action as Bb >> 6 & 1) << sq;
            let simd_road_bit = Simd::splat(road_bit);
            let adjacent = *inf & simd_road_bit;

            // Set the road and noble bits, if appropriate piece.
            let new_road = self.copy.road ^ road_bit;
            new.val.road = new_road;
            new.val.noble = self.copy.noble ^ (action as Bb >> 7 & 1) << sq;

            // Clear and set the owner.
            let new_owner = self.copy.owner & !(1 << sq) | (player as Bb) << sq;
            new.val.owner = new_owner;

            // Placement is irreversible.
            new.val.last_irreversible = self.copy.ply;

            // Set the stack.
            *unsafe { self.update.stacks.get_unchecked_mut(sq as usize) } = player as Stack | 2;

            // Copy the tall bitboard unchanged.
            new.val.tall = self.copy.tall;

            // Check if the new road bit is adjacent to any of the influence masks.
            if adjacent.reduce_or() != 0 {
                // Prepare to add the neighbors to the adjacent masks.
                let neighbors = adjacent
                    .simd_ne(Simd::splat(0))
                    .select(simd_road_bit.neighbors(), Simd::splat(0));

                // Add the neighbors.
                let mut old_inf = *inf;
                *inf |= neighbors; // Codegens a spurious store. Alternatives worse.

                // Get a bitboard representing our road pieces.
                let my_road = Simd::splat(new_road & (new_owner ^ (player as Bb).wrapping_sub(1)));

                // Expandable neighbors are those that are our road pieces.
                let mut expandable = neighbors & my_road;

                // Count iterations for logging purposes.
                let mut i = 0;

                // If we have any such neighbors that are not part of the old influence, expand.
                // Assume winning placements are rare and/or handled before make.
                while (expandable & !old_inf).reduce_or() != 0 {
                    i += 1;
                    old_inf = *inf;
                    *inf |= expandable.neighbors();
                    expandable = *inf & my_road;
                }

                log!(self, PlacementExpansionIterations(i));
            }
        }

        self.update.hashes[new_ply] = hash;
    }
}
