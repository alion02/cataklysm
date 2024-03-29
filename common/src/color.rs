use core::ops::{BitXor, BitXorAssign, Not};

use crate::*;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl From<bool> for Color {
    #[no_mangle]
    #[inline]
    fn from(value: bool) -> Self {
        if value {
            Black
        } else {
            White
        }
    }
}

impl Color {
    #[no_mangle]
    #[inline]
    pub fn sign(self) -> i32 {
        match self {
            White => 1,
            Black => -1,
        }
    }
}

impl Not for Color {
    type Output = Self;

    #[no_mangle]
    #[inline]
    fn not(self) -> Self {
        match self {
            White => Black,
            Black => White,
        }
    }
}

impl BitXor for Color {
    type Output = Self;

    #[no_mangle]
    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        (self as u32 ^ rhs as u32 != 0).into()
    }
}

impl BitXorAssign for Color {
    #[no_mangle]
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}
