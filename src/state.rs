#![allow(unused)]

use std::mem::transmute;

use crate::pair::*;

#[repr(u32)]
enum Piece {
    Flat = 0b01,
    Wall = 0b10,
    Cap = 0b11,
}

#[repr(u32)]
enum Direction {
    Right,
    Up,
    Left,
    Down,
}

mod size6 {
    use super::*;

    // size-dependent parameters

    type Stack = u64;
    type Bitboard = u64;

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct Action(u16);

    const START_STONES: u32 = 30;
    const START_CAPS: u32 = 1;

    const SIZE: usize = 6;
    const ROW_LEN: usize = 8;

    // end of parameters

    const PADDING: usize = ROW_LEN - SIZE;
    const ARR_LEN: usize = SIZE * ROW_LEN - PADDING;

    impl Action {
        const TYPE_OFFSET: u32 = ARR_LEN.ilog2();
        const PAT_OFFSET: u32 = Self::TYPE_OFFSET + 2;

        #[inline(always)]
        fn pass() -> Self {
            Self(0)
        }

        #[inline(always)]
        fn place(sq: usize, piece: Piece) -> Self {
            Self(sq as u16 | (piece as u16) << Self::TYPE_OFFSET)
        }

        #[inline(always)]
        fn spread(sq: usize, dir: Direction, pat: u32) -> Self {
            Self(sq as u16 | (dir as u16) << Self::TYPE_OFFSET | (pat as u16) << Self::PAT_OFFSET)
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
                stacks: [1; ARR_LEN],
            }
        }
    }

    impl State {}
}
