use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Hand(u32);

impl Hand {
    #[inline(always)]
    pub fn one_piece(color: bool) -> Self {
        Self((color as u32).rotate_right(1))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DropCounts(pub u32);

impl fmt::Display for DropCounts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.clone().try_for_each(|count| {
            if count < 10 {
                write!(f, "{}", (b'0' + count as u8) as char)
            } else {
                write!(f, "({count})")
            }
        })
    }
}

impl Iterator for DropCounts {
    type Item = u32;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        (self.0 != 0).then(|| {
            let r = self.0.trailing_zeros();
            self.0 = self.0 >> r & !1;
            r
        })
    }

    #[inline(always)]
    fn count(self) -> usize {
        self.0.count_ones() as usize
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let c = self.count();
        (c, Some(c))
    }

    #[inline(always)]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl DoubleEndedIterator for DropCounts {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        (self.0 != 0).then(|| {
            let t = self.0.leading_zeros();
            self.0 &= !1 << t ^ u32::BITS - 1;
            (self.0 | 1).leading_zeros() - t
        })
    }
}

macro_rules! stack {
    ($mod:ident, $export:ident, $StackBacking:ty) => {
        pub use $mod::Stack as $export;
        mod $mod {
            use super::*;

            type StackBacking = $StackBacking;

            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct Stack(StackBacking);

            impl Default for Stack {
                #[inline(always)]
                fn default() -> Self {
                    Self::EMPTY
                }
            }

            impl Stack {
                pub const EMPTY: Self = Self(1);
                const CAPACITY: u32 = StackBacking::BITS - 1;

                #[inline(always)]
                pub fn one_tall(color: bool) -> Self {
                    Self(0b10 | color as StackBacking)
                }

                #[inline(always)]
                pub fn height(self) -> u32 {
                    self.0.leading_zeros() ^ StackBacking::BITS - 1
                }

                #[inline(always)]
                pub fn is_empty(self) -> bool {
                    self == Stack::EMPTY
                }

                #[inline(always)]
                pub fn drop(&mut self, hand: &mut Hand, count: u32) {
                    debug_assert!(count != 0);
                    debug_assert!(self.height() + count <= Stack::CAPACITY);

                    self.0 =
                        self.0 << count | hand.0.wrapping_shr(count.wrapping_neg()) as StackBacking;
                    hand.0 <<= count;
                }

                #[inline(always)]
                pub fn take(&mut self, count: u32) -> Hand {
                    debug_assert!(count != 0);
                    debug_assert!(self.height() >= count);

                    let r = Hand((self.0 as u32).wrapping_shl(count.wrapping_neg()));
                    self.0 >>= count;
                    r
                }

                #[inline(always)]
                pub fn top(self) -> Option<bool> {
                    (!self.is_empty()).then_some(self.0 & 1 != 0)
                }

                #[inline(always)]
                pub fn top_unchecked(self) -> bool {
                    debug_assert!(!self.is_empty());
                    self.0 & 1 != 0
                }
            }
        }
    };
}

stack!(capacity16, Stack16, u16);
stack!(capacity32, Stack32, u32);
stack!(capacity64, Stack64, u64);
stack!(capacity128, Stack128, u128);
