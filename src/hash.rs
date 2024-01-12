use rand::{
    distributions::{Distribution, Standard},
    prelude::Rng,
};

pub const HIST_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hash(u64);

impl Hash {
    pub const ZERO: Self = Self(0);

    #[inline(always)]
    pub fn xor(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }
}

impl Distribution<Hash> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Hash {
        Hash(rng.gen())
    }
}
