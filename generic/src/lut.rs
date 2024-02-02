use crate::prelude::*;

use std::sync::Mutex;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng; // NOTE: Requires std with default features

static INIT: Mutex<bool> = Mutex::new(false);

pub static mut HASH_WALL: [Hash; ARR_LEN] = [Hash::ZERO; ARR_LEN];
pub static mut HASH_CAP: [Hash; ARR_LEN] = [Hash::ZERO; ARR_LEN];
pub static mut HASH_STACK: [[[Hash; 2 << HAND]; Stack::CAPACITY as usize]; ARR_LEN] =
    [[[Hash::ZERO; 2 << HAND]; Stack::CAPACITY as usize]; ARR_LEN];

pub fn init() {
    let mut init = INIT.lock().unwrap();
    if !*init {
        let mut rng = ChaCha20Rng::seed_from_u64(0);

        unsafe {
            HASH_WALL.iter_mut().for_each(|h| *h = rng.gen());
            HASH_CAP.iter_mut().for_each(|h| *h = rng.gen());

            #[allow(clippy::needless_range_loop)]
            for i in 0..HASH_STACK.len() {
                for k in 2..HASH_STACK[0][0].len() {
                    // 0..2 unused
                    for j in 0..HASH_STACK[0].len() {
                        // Iterate over heights first (patterns combine heights)
                        HASH_STACK[i][j][k] = if k < 4 {
                            // Generate hash value for the given square-height-color
                            rng.gen()
                        } else {
                            // Combine hashes
                            let mut stack = Stack::from_raw(k as _);
                            let top = Stack::from_hand_and_count(stack.take(1), 1);

                            let top_j = j + stack.height() as usize;
                            if top_j >= HASH_STACK[i].len() {
                                continue;
                            }

                            HASH_STACK[i][j][stack.raw() as usize]
                                ^ HASH_STACK[i][top_j][top.raw() as usize]
                        };
                    }
                }
            }
        }

        *init = true;
    }
}
