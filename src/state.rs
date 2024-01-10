use std::{fmt, mem::transmute, ops::Neg};

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Flat = 0b01,
    Wall = 0b10,
    Cap = 0b11,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Flat => "",
            Self::Wall => "S",
            Self::Cap => "C",
        })
    }
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

    #[inline(always)]
    pub fn is_flat(self) -> bool {
        self == Self::Flat
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

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Right => ">",
            Self::Up => "+",
            Self::Left => "<",
            Self::Down => "-",
        })
    }
}

impl Neg for Direction {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        // Generates a lookup table
        // match self {
        //     Self::Right => Self::Left,
        //     Self::Up => Self::Down,
        //     Self::Left => Self::Right,
        //     Self::Down => Self::Up,
        // }

        unsafe { transmute(self as u32 ^ 2) }
    }
}

include!(concat!(env!("OUT_DIR"), "/macro.rs"));

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
