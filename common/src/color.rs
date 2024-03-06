use core::{mem::transmute, ops::Not};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[no_mangle]
    #[inline]
    pub fn sign(self) -> i32 {
        // Better standalone, but optimizes worse when used with multiplication (common use case):
        // 1 - self as i32 * 2

        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }
}

impl Not for Color {
    type Output = Self;

    #[no_mangle]
    #[inline]
    fn not(self) -> Self {
        // TODO: Investigate optimization quality
        // match self {
        // 	Color::White => Color::Black,
        // 	Color::Black => Color::White,
        // }

        unsafe { transmute(self as u32 ^ 1) }
    }
}
