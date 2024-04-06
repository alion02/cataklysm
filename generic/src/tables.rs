use rand::{Rng, SeedableRng};

use crate::*;

const HASH_STACK_LEN: usize = ((2 << HAND) - 2) * (STACK_CAP + 1) * ARR_LEN;
static mut HASH_SQ_PC: [u64; 1 << PAT_OFFSET] = [0; 1 << PAT_OFFSET];
static mut HASH_STACK: [u64; HASH_STACK_LEN] = [0; HASH_STACK_LEN];

// TODO: LLVM happily loads the addresses of the tables on every access for some reason. Investigate.

#[inline]
pub fn hash_sq_pc(i: usize) -> u64 {
    unsafe {
        debug_assert!(i < HASH_SQ_PC.len());
        *HASH_SQ_PC.get_unchecked(i)
    }
}

#[inline]
fn hash_stack_index(sq: usize, stack: u32, rem_cap: u32) -> usize {
    sq + rem_cap as usize * ARR_LEN + (stack as usize - 2) * ARR_LEN * (STACK_CAP + 1)
}

#[inline]
pub fn hash_stack(sq: usize, stack: u32, rem_cap: u32) -> u64 {
    let i = hash_stack_index(sq, stack, rem_cap);
    unsafe {
        debug_assert!(i < HASH_STACK.len());
        *HASH_STACK.get_unchecked(i)
    }
}

pub fn init() {
    // TODO: Upgrade to Mutex<bool> if feature = "std"
    static INIT_STAGE: AtomicU8 = AtomicU8::new(0);

    while let Err(stage) = INIT_STAGE.compare_exchange_weak(0, 1, Relaxed, Acquire) {
        if stage == 2 {
            return;
        }

        core::hint::spin_loop();
    }

    // TODO: alt-seed
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(0);

    let mut left = BOARD;
    while left > 0 {
        let sq = left.trailing_zeros() as u16;

        // SAFETY: No other references to the tables exist right now. We are guarding against
        // simultaneous initialization with a spin loop, repeated initialization with an early
        // exit, and all objects which use tables are assumed to have called this first.
        unsafe {
            HASH_SQ_PC[(sq | WALL_TAG << TAG_OFFSET) as usize] = rng.gen();
            HASH_SQ_PC[(sq | CAP_TAG << TAG_OFFSET) as usize] = rng.gen();

            let sq = sq as usize;

            for stack in 2u32..2 << HAND {
                let height = stack.ilog2() as usize;
                for rem_cap in height..STACK_CAP + 1 {
                    HASH_STACK[hash_stack_index(sq, stack, rem_cap as _)] = if stack < 1 << 2 {
                        // Generate hash for the given square-color-capacity triplet.
                        rng.gen()
                    } else {
                        // Create a hash for a sequence of colors by combining pairs of previously
                        // computed hashes.

                        // Split the stack into the top color and the rest.
                        let top = stack & 1 + 2;
                        let rest = stack >> 1;
                        hash_stack(sq, top, (rem_cap - height + 1) as _)
                            ^ hash_stack(sq, rest, rem_cap as _)
                    };
                }
            }
        }

        left &= left - 1;
    }

    INIT_STAGE.store(2, Release);
}
