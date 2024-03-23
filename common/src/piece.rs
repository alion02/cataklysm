use crate::*;

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Flat = 1,
    Wall = 2,
    Cap = 3,
}

// impl Piece {
//     #[no_mangle]
//     #[inline]
//     pub fn is_road(self) -> bool {
//         self as u32 & 1 != 0
//     }

//     #[no_mangle]
//     #[inline]
//     pub fn is_noble(self) -> bool {
//         self as u32 & 2 != 0
//     }

//     #[no_mangle]
//     #[inline]
//     pub fn is_stone(self) -> bool {
//         self != Cap
//     }

//     #[no_mangle]
//     #[inline]
//     pub fn is_flat(self) -> bool {
//         self as u32 & 2 == 0
//     }

//     #[no_mangle]
//     #[inline]
//     pub fn is_wall(self) -> bool {
//         self as u32 & 1 == 0
//     }

//     #[no_mangle]
//     #[inline]
//     pub fn is_cap(self) -> bool {
//         self == Cap
//     }
// }

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Flat => "",
            Wall => "S",
            Cap => "C",
        })
    }
}
