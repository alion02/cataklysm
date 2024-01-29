use std::{
    convert::Infallible,
    ops::{ControlFlow, Index, IndexMut},
};

pub trait ControlFlowIntoContinue<T> {
    fn into_continue(self) -> T;
}

impl<T> ControlFlowIntoContinue<T> for ControlFlow<Infallible, T> {
    #[inline]
    fn into_continue(self) -> T {
        match self {
            ControlFlow::Continue(v) => v,
            ControlFlow::Break(v) => match v {},
        }
    }
}

pub trait ControlFlowIntoInner<T> {
    fn into_inner(self) -> T;
}

impl<T> ControlFlowIntoInner<T> for ControlFlow<T, T> {
    #[inline]
    fn into_inner(self) -> T {
        match self {
            ControlFlow::Continue(v) => v,
            ControlFlow::Break(v) => v,
        }
    }
}

// Default only available for arrays up to length 32 as of the date of this commit
// #[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct WrappingArray<T, const S: usize>(pub [T; S]);

impl<T, const S: usize> Index<u32> for WrappingArray<T, S> {
    type Output = T;

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        &self.0[index as usize % S]
    }
}

impl<T, const S: usize> IndexMut<u32> for WrappingArray<T, S> {
    #[inline]
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.0[index as usize % S]
    }
}

macro_rules! bits {
    ($Bits:ident, $uint:ty) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct $Bits<const N: usize>([$uint; N]);

        impl<const N: usize> $Bits<N> {
            #[inline]
            pub fn new(masks: [$uint; N]) -> Self {
                Self(masks)
            }
        }

        impl<const N: usize> Iterator for $Bits<N> {
            type Item = u32;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.0.iter_mut().zip(0..).find_map(|(m, i)| {
                    (*m != 0).then(|| (i * <$uint>::BITS + m.trailing_zeros(), *m &= *m - 1).0)
                })
            }

            #[inline]
            fn count(self) -> usize {
                self.0.into_iter().fold(0, |acc, m| acc + m.count_ones()) as usize
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let c = self.count();
                (c, Some(c))
            }
        }
    };
}

bits!(Bits8, u8);
bits!(Bits16, u16);
bits!(Bits32, u32);
bits!(Bits64, u64);
bits!(Bits128, u128);
