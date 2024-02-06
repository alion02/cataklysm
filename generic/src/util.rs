use crate::*;

pub trait BitboardExt {
    fn shift(self, dir: Direction) -> Self;
    fn spread(self) -> Self;
}

impl BitboardExt for Bitboard {
    fn shift(self, dir: Direction) -> Self {
        match dir {
            Right => self << 1 & !if PADDING != 0 { 0 } else { COL },
            Up => self << ROW_LEN,
            Left => self >> 1 & !if PADDING != 0 { 0 } else { COL << SIZE - 1 },
            Down => self >> ROW_LEN,
        }
    }

    fn spread(self) -> Self {
        self.shift(Right) | self.shift(Left) | self.shift(Up) | self.shift(Down)
    }
}

pub fn ray(src: Square, dir: Direction) -> Bitboard {
    match dir {
        Right => src.row_bitboard() & !1 << src.0,
        Up => src.col_bitboard() & !1 << src.0,
        Left => src.row_bitboard() & (1 << src.0) - 1,
        Down => src.col_bitboard() & (1 << src.0) - 1,
    }
}

pub fn closest_hit(ray_hits: Bitboard, dir: Direction) -> Bitboard {
    ray_hits
        & match dir {
            Right | Up => ray_hits.wrapping_neg(),
            Left | Down => !(!(0 as Bitboard) >> 1).wrapping_shr(ray_hits.leading_zeros()),
        }
}

pub fn distance(src: Square, hit: Square, dir: Direction) -> u32 {
    (match dir {
        Right => hit.0 - src.0,
        Up => (hit.0 - src.0) / ROW_LEN,
        Left => src.0 - hit.0,
        Down => (src.0 - hit.0) / ROW_LEN,
    }) as u32
}

pub fn bit_squares(bitboard: Bitboard) -> impl Iterator<Item = Square> {
    Bits::new([bitboard]).map(|s| sq(s as usize))
}