use core::ops::{BitXor, BitXorAssign};

use rand::{distributions::Standard, prelude::*};

// Symmetry-supporting hashes considered but ultimately rejected due to their tendency to infect
// other parts of the codebase, implementation complexity, and lack of general usefulness.

pub const DEFAULT_SEED: u64 = cfg!(feature = "alt-seed") as u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hash(u64);

impl Hash {
    pub const ZERO: Self = Self(0);
    pub const SIDE_TO_MOVE: Self = Self(if cfg!(feature = "alt-seed") {
        0x1815ad0c9e50c110u64
    } else {
        0xf812ec2e34a9c388u64
    });

    /// # Panics
    ///
    /// The method may panic if the `len` provided is zero, not a power of two, or too large.
    #[no_mangle]
    #[inline]
    pub fn split(self, len: usize) -> SplitHash {
        debug_assert!(len.is_power_of_two());
        debug_assert!(len.trailing_zeros() <= u32::BITS);
        SplitHash {
            idx: self.0 as u32 & (len - 1) as u32,
            sig: (self.0 >> 32) as u32,
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitHash {
    pub idx: u32,
    pub sig: u32,
}

impl Distribution<Hash> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Hash {
        Hash(rng.gen())
    }
}
