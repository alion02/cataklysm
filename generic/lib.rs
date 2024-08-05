#![feature(portable_simd)]
#![allow(
    clippy::precedence, // Preference
    unused, // TODO: Remove when appropriate
)]

mod params;

use params::*;

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

const FLAT_TAG: u32 = 1;
const WALL_TAG: u32 = 2;
const CAP_TAG: u32 = 3;
