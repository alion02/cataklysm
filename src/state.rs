#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Flat = 0b01,
    Wall = 0b10,
    Cap = 0b11,
}

impl Piece {
    #[inline(always)]
    pub fn is_road(self) -> bool {
        self as u32 & 1 != 0
    }

    #[inline(always)]
    pub fn is_block(self) -> bool {
        self as u32 & 2 != 0
    }

    #[inline(always)]
    pub fn is_stone(self) -> bool {
        self != Self::Cap
    }
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
}

include!(concat!(env!("OUT_DIR"), "/macro.rs"));

#[cfg(feature = "3")]
state!(size3 => State3 {
    size: 3,
    row_len: 4,
    bitboard: u16,
    stack: Stack32,
    action: u16,
    perft: [
        (1, 9),
        (2, 72),
        (3, 1200),
        (4, 17792),
        (5, 271812),
        (6, 3715592),
        // (7, 52400728),
    ],
});

#[cfg(feature = "4")]
state!(size4 => State4 {
    size: 4,
    row_len: 5, // TODO
    bitboard: u32,
    stack: Stack32,
    action: u16,
    perft: [
        (1, 16),
        (2, 240),
        (3, 7440),
        (4, 216464),
        (5, 6468872),
        // (6, 181954216),
    ],
});

#[cfg(feature = "5")]
state!(size5 => State5 {
    size: 5,
    row_len: 6,
    bitboard: u32,
    stack: Stack64,
    action: u16,
    perft: [
        (1, 25),
        (2, 600),
        (3, 43320),
        (4, 2999784),
        (5, 187855252),
    ],
});

#[cfg(feature = "6")]
state!(size6 => State6 {
    size: 6,
    row_len: 8,
    bitboard: u64,
    stack: Stack64,
    action: u16,
    perft: [
        (1, 36),
        (2, 1260),
        (3, 132720),
        (4, 13586048),
        // (5, 1253506520),
        // (6, 112449385016),
    ],
});

#[cfg(feature = "7")]
state!(size7 => State7 {
    size: 7,
    row_len: 8,
    bitboard: u64,
    stack: Stack128,
    action: u16,
    perft: [
        (1, 49),
        (2, 2352),
        (3, 339696),
        (4, 48051008),
    ],
});

#[cfg(feature = "8")]
state!(size8 => State8 {
    size: 8,
    row_len: 9, // TODO
    bitboard: u128,
    stack: Stack128,
    action: u32,
    perft: [
        (1, 64),
        (2, 4032),
        (3, 764064),
        (4, 142512336),
    ],
});
