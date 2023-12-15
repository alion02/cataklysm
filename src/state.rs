#![allow(unused)]

use std::{
    mem::transmute,
    ops::{Index, IndexMut},
};

use crate::pair::*;
use crate::stack::*;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Piece {
    Flat = 0b01,
    Wall = 0b10,
    Cap = 0b11,
}

impl Piece {
    #[inline(always)]
    fn is_road(self) -> bool {
        self as u32 & 1 != 0
    }

    #[inline(always)]
    fn is_block(self) -> bool {
        self as u32 & 2 != 0
    }

    #[inline(always)]
    fn is_stone(self) -> bool {
        self != Self::Cap
    }
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Right,
    Up,
    Left,
    Down,
}

mod size6 {
    use super::*;

    // size-dependent parameters

    type Bitboard = u64;
    type Stack = Stack64;
    type ActionBacking = u16;

    const START_STONES: u32 = 30;
    const START_CAPS: u32 = 1;

    const SIZE: usize = 6;
    const ROW_LEN: usize = 8;

    // end of parameters

    const HAND: u32 = SIZE as u32;

    const PADDING: usize = ROW_LEN - SIZE;
    const ARR_LEN: usize = SIZE * ROW_LEN - PADDING;

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct Square(usize);

    impl<T> Index<Square> for [T] {
        type Output = T;

        #[inline(always)]
        fn index(&self, index: Square) -> &Self::Output {
            unsafe { self.get(index.0).unwrap_unchecked() }
        }
    }

    impl<T> IndexMut<Square> for [T] {
        #[inline(always)]
        fn index_mut(&mut self, index: Square) -> &mut Self::Output {
            unsafe { self.get_mut(index.0).unwrap_unchecked() }
        }
    }

    impl Square {
        #[inline(always)]
        fn bit(self) -> Bitboard {
            1 << self.0
        }
    }

    #[inline(always)]
    fn sq(sq: usize) -> Square {
        debug_assert!(sq < ARR_LEN);
        debug_assert!(sq % ROW_LEN < SIZE);

        Square(sq)
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct Pattern(u32);

    impl Pattern {
        #[inline(always)]
        fn execute(self) -> (u32, DropCounts) {
            let mut dc = DropCounts(self.0 | 1 << HAND);
            (HAND - dc.next().unwrap(), dc)
        }
    }

    #[inline(always)]
    fn pat(pat: u32) -> Pattern {
        debug_assert!(pat > 0);
        debug_assert!(pat < 1 << HAND);

        Pattern(pat)
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct Action(ActionBacking);

    impl Action {
        const TYPE_OFFSET: u32 = ARR_LEN.ilog2();
        const PAT_OFFSET: u32 = Self::TYPE_OFFSET + 2;

        #[inline(always)]
        fn pass() -> Self {
            Self(0)
        }

        #[inline(always)]
        fn place(sq: Square, piece: Piece) -> Self {
            Self(sq.0 as ActionBacking | (piece as ActionBacking) << Self::TYPE_OFFSET)
        }

        #[inline(always)]
        fn spread(sq: Square, dir: Direction, pat: Pattern) -> Self {
            Self(
                sq.0 as ActionBacking
                    | (dir as ActionBacking) << Self::TYPE_OFFSET
                    | (pat.0 as ActionBacking) << Self::PAT_OFFSET,
            )
        }

        #[inline(always)]
        fn branch<S, R>(
            self,
            state: S,
            pass: impl FnOnce(S) -> R,
            place: impl FnOnce(S, Square, Piece) -> R,
            spread: impl FnOnce(S, Square, Direction, Pattern) -> R,
        ) -> R {
            if self.0 == 0 {
                pass(state)
            } else {
                let sq = sq(self.0 as usize & (1 << Self::TYPE_OFFSET) - 1);
                if self.0 < 1 << Self::PAT_OFFSET {
                    place(state, sq, unsafe {
                        transmute(self.0 as u32 >> Self::TYPE_OFFSET)
                    })
                } else {
                    spread(
                        state,
                        sq,
                        unsafe { transmute(self.0 as u32 >> Self::TYPE_OFFSET & 3) },
                        pat(self.0 as u32 >> Self::PAT_OFFSET),
                    )
                }
            }
        }
    }

    #[repr(C)]
    pub struct State {
        road: Pair<Bitboard>,
        block: Pair<Bitboard>,

        stones_left: Pair<u32>,
        caps_left: Pair<u32>,

        ply: u32,

        stacks: [Stack; ARR_LEN],
    }

    impl Default for State {
        fn default() -> Self {
            Self {
                road: Pair::default(),
                block: Pair::default(),
                stones_left: Pair::both(START_STONES),
                caps_left: Pair::both(START_CAPS),
                ply: 0,
                stacks: [Stack::EMPTY; ARR_LEN],
            }
        }
    }

    impl State {
        #[inline(always)]
        fn color(&self) -> bool {
            self.ply & 1 != 0
        }

        fn with<R>(&mut self, undo: bool, action: Action, f: impl FnOnce(&mut Self) -> R) -> R {
            let s = self;
            let color = s.color();

            s.ply += 1;

            let (s, r) = action.branch(
                (s, f),
                |(s, f)| {
                    let r = f(s);
                    (s, r)
                },
                |(s, f), sq, piece| {
                    let bit = sq.bit();

                    if piece.is_road() {
                        s.road[color] ^= bit;
                    }

                    if piece.is_block() {
                        s.block[color] ^= bit;
                    }

                    if piece.is_stone() {
                        s.stones_left[color] -= 1;
                    } else {
                        s.caps_left[color] -= 1;
                    }

                    s.stacks[sq] = Stack::one_tall(color);

                    let r = f(s);

                    if undo {
                        s.stacks[sq] = Stack::EMPTY;

                        if piece.is_stone() {
                            s.stones_left[color] += 1;
                        } else {
                            s.caps_left[color] += 1;
                        }

                        if piece.is_block() {
                            s.block[color] ^= bit;
                        }

                        if piece.is_road() {
                            s.road[color] ^= bit;
                        }
                    }

                    (s, r)
                },
                |(s, f), sq, dir, mut pat| {
                    let bit = sq.bit();

                    let road = s.road[color];
                    let block = s.block[color];
                    let stacks = s.stacks;

                    let (taken, drops) = pat.execute();

                    let (mut hand, empty) = s.stacks[sq.0].pop(taken);

                    let r = f(s);

                    if undo {
                        s.stacks = stacks;
                        s.block[color] = block;
                        s.road[color] = road;
                    }

                    todo!()
                },
            );

            if undo {
                s.ply -= 1;
            }

            r
        }
    }
}
