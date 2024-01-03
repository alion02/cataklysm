use std::{convert::Infallible, ops::ControlFlow};

pub trait ControlFlowIntoContinue<T> {
    fn into_continue(self) -> T;
}

impl<T> ControlFlowIntoContinue<T> for ControlFlow<Infallible, T> {
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
    fn into_inner(self) -> T {
        match self {
            ControlFlow::Continue(v) => v,
            ControlFlow::Break(v) => v,
        }
    }
}

macro_rules! bits {
    ($Bits:ident, $uint:ty) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct $Bits<const N: usize>([$uint; N]);

        impl<const N: usize> $Bits<N> {
            pub fn new(masks: [$uint; N]) -> Self {
                Self(masks)
            }
        }

        impl<const N: usize> Iterator for $Bits<N> {
            type Item = u32;

            fn next(&mut self) -> Option<Self::Item> {
                self.0.iter_mut().zip(0..).find_map(|(m, i)| {
                    (*m != 0).then(|| (i * <$uint>::BITS + m.trailing_zeros(), *m &= *m - 1).0)
                })
            }

            fn count(self) -> usize {
                self.0.into_iter().fold(0, |acc, m| acc + m.count_ones()) as usize
            }

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
