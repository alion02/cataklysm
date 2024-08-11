pub use rand_chacha::rand_core::RngCore;
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};

pub fn rng() -> impl RngCore {
    ChaCha20Rng::seed_from_u64(cfg!(feature = "alt-seed") as u64)
}
