use rand::SeedableRng;

use crate::*;

#[allow(clippy::declare_interior_mutable_const)] // False positive.
const ATOMIC_U64_ZERO: AtomicU64 = AtomicU64::new(0);

pub static HASH_PC_SQ: [AtomicU64; 1 << PAT_OFFSET] = [ATOMIC_U64_ZERO; 1 << PAT_OFFSET];
static HASH_STACK: [AtomicU64; ((2 << HAND) - 2) * STACK_CAP * ARR_LEN] =
    [ATOMIC_U64_ZERO; ((2 << HAND) - 2) * STACK_CAP * ARR_LEN];

#[inline]
pub fn hash_stack(sq: u16, stack: Stack, rem_cap: u32) -> &'static AtomicU64 {
    let i = sq as usize + rem_cap as usize * ARR_LEN + (stack as usize - 2) * ARR_LEN * STACK_CAP;
    #[cfg(debug_assertions)]
    {
        &HASH_STACK[i]
    }
    #[cfg(not(debug_assertions))]
    unsafe {
        HASH_STACK.get_unchecked(i)
    }
}

pub fn init() {
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(0);

    let mut left = BOARD;
    while left > 0 {
        let sq = left.trailing_zeros() as u16;
        left &= left - 1;
    }
}
