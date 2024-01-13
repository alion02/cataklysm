use std::ops::{BitXor, BitXorAssign};

use rand::{
    distributions::{Distribution, Standard},
    prelude::Rng,
};

pub const HIST_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hash(u64);

impl Hash {
    pub const ZERO: Self = Self(0);
    pub const SIDE_TO_MOVE: Self = Self(0xf812ec2e34a9c388); // 1815ad0c9e50c110
}

impl BitXor for Hash {
    type Output = Hash;

    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Hash {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Distribution<Hash> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Hash {
        Hash(rng.gen())
    }
}
