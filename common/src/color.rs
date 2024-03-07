use crate::*;

use core::ops::Not;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 1,
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
