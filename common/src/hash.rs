use core::ops::{BitXor, BitXorAssign};

use rand::{
    distributions::{Distribution, Standard},
    prelude::Rng,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hash(u64);

impl Hash {
    pub const ZERO: Self = Self(0);
    pub const SIDE_TO_MOVE: Self = Self(0xf812ec2e34a9c388); // 1815ad0c9e50c110

    /// # Panics
    ///
    /// The method may panic if the `len` provided is zero or is not a power of two.
    #[inline]
    pub fn split(self, len: usize) -> (usize, u64) {
        debug_assert!(len.is_power_of_two());
        (self.0 as usize & len - 1, self.0)
    }
}

impl BitXor for Hash {
    type Output = Hash;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Hash {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Distribution<Hash> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Hash {
        Hash(rng.gen())
    }
}
