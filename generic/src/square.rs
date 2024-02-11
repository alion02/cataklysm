use crate::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Square(pub usize);

#[must_use]
#[inline]
pub const fn sq(sq: usize) -> Square {
    debug_assert!(sq < ARR_LEN);
    debug_assert!(sq % ROW_LEN < SIZE);

    Square(sq)
}

impl Square {
    #[must_use]
    #[inline]
    pub fn bit(self) -> Bitboard {
        1 << self.0
    }

    #[must_use]
    #[inline]
    pub fn shift(self, amount: usize, dir: Direction) -> Self {
        sq(match dir {
            Right => self.0 + amount,
            Up => self.0 + amount * ROW_LEN,
            Left => self.0 - amount,
            Down => self.0 - amount * ROW_LEN,
        })
    }

    #[must_use]
    #[inline]
    pub const fn row_bitboard(self) -> Bitboard {
        ROW << self.align_left().0
    }

    #[must_use]
    #[inline]
    pub const fn col_bitboard(self) -> Bitboard {
        COL << self.align_bottom().0
    }

    #[must_use]
    #[inline]
    pub const fn row(self) -> usize {
        self.0 / ROW_LEN
    }

    #[must_use]
    #[inline]
    pub const fn col(self) -> usize {
        self.0 % ROW_LEN
    }

    #[must_use]
    #[inline]
    pub const fn align_left(self) -> Self {
        sq(self.row() * ROW_LEN)
    }

    #[must_use]
    #[inline]
    pub const fn align_bottom(self) -> Self {
        sq(self.col())
    }
}

impl<T> Index<Square> for [T] {
    type Output = T;

    #[inline]
    fn index(&self, index: Square) -> &Self::Output {
        unsafe { self.get(index.0).unwrap_unchecked() }
    }
}

impl<T> IndexMut<Square> for [T] {
    #[inline]
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        unsafe { self.get_mut(index.0).unwrap_unchecked() }
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", (b'a' + self.col() as u8) as char, self.row() + 1,)
    }
}
