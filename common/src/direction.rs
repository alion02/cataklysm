use crate::*;

use core::{fmt, mem::transmute, ops::Neg};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
}

impl Neg for Direction {
    type Output = Self;

    #[no_mangle]
    #[inline]
    fn neg(self) -> Self::Output {
        // Generates a lookup table
        // match self {
        //     Right => Left,
        //     Up => Down,
        //     Left => Right,
        //     Down => Up,
        // }

        unsafe { transmute(self as u8 ^ 2) }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Right => ">",
            Up => "+",
            Left => "<",
            Down => "-",
        })
    }
}
