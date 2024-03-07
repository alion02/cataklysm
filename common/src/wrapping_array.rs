use core::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// Default only available for arrays up to length 32 as of the date of this commit
// #[derive(Default)]
pub struct WrappingArray<T, const S: usize>(pub [T; S]);

impl<T, I: Into<usize>, const S: usize> Index<I> for WrappingArray<T, S> {
    type Output = T;

    #[inline]
    fn index(&self, index: I) -> &T {
        &self.0[index.into() % S]
    }
}

impl<T, I: Into<usize>, const S: usize> IndexMut<I> for WrappingArray<T, S> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut T {
        &mut self.0[index.into() % S]
    }
}
