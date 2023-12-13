use crate::pair::*;

#[allow(unused)]
mod size6 {
    use super::*;

    const START_STONES: u32 = 30;
    const START_CAPS: u32 = 1;

    type Stack = u64;
    type Bitboard = u64;

    const SIZE: usize = 6;
    const ROW_LEN: usize = 8;

    const PADDING: usize = ROW_LEN - SIZE;
    const ARR_LEN: usize = SIZE * ROW_LEN - PADDING;

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
