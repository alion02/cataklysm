#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Hand(u32);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pattern<const HAND: u32>(u32);

impl<const HAND: u32> Pattern<HAND> {
    #[inline(always)]
    pub fn new(mask: u32) -> Self {
        assert!(HAND < u32::BITS);

        debug_assert!(mask > 0);
        debug_assert!(mask < 1 << HAND);

        Self(mask)
    }

    #[inline(always)]
    pub fn drop_counts(self) -> (u32, DropCounts) {
        let mut dc = DropCounts(self.0 | 1 << HAND);
        (HAND - dc.next().unwrap(), dc)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DropCounts(u32);

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
}

mod stack64 {
    use super::*;

    type StackBacking = u64;

    #[derive(Clone, Copy, PartialEq, Eq)]
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
            self.0 == 1
        }

        #[inline(always)]
        pub fn push(&mut self, hand: &mut Hand, count: u32) {
            debug_assert!(count != 0);
            debug_assert!(self.height() + count <= Stack::CAPACITY);

            self.0 = self.0 << count | hand.0.wrapping_shr(count.wrapping_neg()) as StackBacking;
            hand.0 <<= count;
        }

        #[inline(always)]
        pub fn pop(&mut self, count: u32) -> (Hand, bool) {
            debug_assert!(count != 0);
            debug_assert!(self.height() >= count);

            let r = Hand((self.0 as u32).wrapping_shl(count.wrapping_neg()));
            self.0 >>= count;
            (r, self.is_empty())
        }
    }
}

pub use stack64::Stack as Stack64;
