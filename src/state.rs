#![allow(unused)]

use std::mem::transmute;

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

    type Pat = Pattern<HAND>;

    const HAND: u32 = SIZE as u32;

    const PADDING: usize = ROW_LEN - SIZE;
    const ARR_LEN: usize = SIZE * ROW_LEN - PADDING;

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
        fn place(sq: usize, piece: Piece) -> Self {
            Self(sq as ActionBacking | (piece as ActionBacking) << Self::TYPE_OFFSET)
        }

        #[inline(always)]
        fn spread(sq: usize, dir: Direction, pat: u32) -> Self {
            Self(
                sq as ActionBacking
                    | (dir as ActionBacking) << Self::TYPE_OFFSET
                    | (pat as ActionBacking) << Self::PAT_OFFSET,
            )
        }

        #[inline(always)]
        fn branch<S, R>(
            self,
            state: S,
            pass: impl FnOnce(S) -> R,
            place: impl FnOnce(S, usize, Piece) -> R,
            spread: impl FnOnce(S, usize, Direction, u32) -> R,
        ) -> R {
            if self.0 == 0 {
                pass(state)
            } else {
                let sq = self.0 as usize & (1 << Self::TYPE_OFFSET) - 1;
                if self.0 < 1 << Self::PAT_OFFSET {
                    place(state, sq, unsafe {
                        transmute(self.0 as u32 >> Self::TYPE_OFFSET)
                    })
                } else {
                    spread(
                        state,
                        sq,
                        unsafe { transmute(self.0 as u32 >> Self::TYPE_OFFSET & 3) },
                        self.0 as u32 >> Self::PAT_OFFSET,
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
                    let bit = 1 << sq;

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
                    let bit = 1 << sq;

                    let road = s.road[color];
                    let block = s.block[color];
                    let stacks = s.stacks;

                    let (mut hand, empty) = s.stacks[sq].pop(HAND - pat.trailing_zeros());

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
