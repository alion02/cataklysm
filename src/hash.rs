pub const HIST_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hash(u64);

impl Hash {
    #[inline(always)]
    pub fn xor(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }
}
