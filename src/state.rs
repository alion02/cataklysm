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

pub const EVAL_DECISIVE: i32 = 1 << 16;
pub const EVAL_MAX: i32 = 1 << 24;

#[cfg(feature = "3")]
state!(size3 => State3);

#[cfg(feature = "4")]
state!(size4 => State4);

#[cfg(feature = "5")]
state!(size5 => State5);

#[cfg(feature = "6")]
state!(size6 => State6);

#[cfg(feature = "7")]
state!(size7 => State7);

#[cfg(feature = "8")]
state!(size8 => State8);
