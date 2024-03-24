use crate::*;

pub trait Bitboard {
    /// Returns a bitboard containing the neighbors of set tiles. This doesn't necessarily include
    /// all the original tiles -- see [`Neighbors::or_neighbors`] if that is desired. The returned
    /// bitboard may have bits spuriously set outside the [`BOARD`].
    #[must_use]
    fn neighbors(self) -> Self;

    /// Returns a bitboard containing the neighbors of set tiles, including themselves. The
    /// returned bitboard may have bits spuriously set outside the [`BOARD`].
    #[must_use]
    fn or_neighbors(self) -> Self;

    fn expand(&mut self, mask: Self);
}

impl Bitboard for Bb {
    #[inline]
    fn neighbors(self) -> Self {
        if PADDING > 0 {
            self << 1 | self >> 1 | self << ROW_LEN | self >> ROW_LEN
        } else {
            self << 1 & !EDGE_LEFT | self >> 1 & !EDGE_RIGHT | self << ROW_LEN | self >> ROW_LEN
        }
    }

    #[inline]
    fn or_neighbors(self) -> Self {
        self | self.neighbors()
    }

    #[inline]
    fn expand(&mut self, mask: Self) {
        todo!()
    }
}

impl<const N: usize> Bitboard for Simd<Bb, N>
where
    LaneCount<N>: SupportedLaneCount,
{
    #[inline]
    fn neighbors(self) -> Self {
        if PADDING > 0 {
            self << 1 | self >> 1 | self << ROW_LEN as Bb | self >> ROW_LEN as Bb
        } else {
            self << 1 & Simd::splat(!EDGE_LEFT)
                | self >> 1 & Simd::splat(!EDGE_RIGHT)
                | self << ROW_LEN as Bb
                | self >> ROW_LEN as Bb
        }
    }

    #[inline]
    fn or_neighbors(self) -> Self {
        self | self.neighbors()
    }

    #[inline]
    fn expand(&mut self, mask: Self) {
        todo!()
    }
}
