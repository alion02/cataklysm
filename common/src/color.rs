use core::{
    mem::transmute,
    ops::{BitXor, BitXorAssign},
};

use crate::*;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl From<bool> for Color {
    #[inline]
    fn from(value: bool) -> Self {
        if value {
            Black
        } else {
            White
        }
    }
}

// impl Color {
//     #[inline]
//     pub fn sign(self) -> i32 {
//         match self {
//             White => 1,
//             Black => -1,
//         }
//     }
// }

// impl Not for Color {
//     type Output = Self;

//     #[inline]
//     fn not(self) -> Self {
//         match self {
//             White => Black,
//             Black => White,
//         }
//     }
// }

impl BitXor<bool> for Color {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: bool) -> Self {
        unsafe { transmute(self as u32 ^ rhs as u32) }
    }
}

impl BitXorAssign<bool> for Color {
    #[inline]
    fn bitxor_assign(&mut self, rhs: bool) {
        *self = *self ^ rhs;
    }
}
