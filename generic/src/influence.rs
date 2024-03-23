use crate::*;

#[repr(C)]
pub union Influence {
    pub pair: Pair<Simd<Bb, 4>>,
    pub vec: Simd<Bb, 8>,
}

impl Influence {
    #[inline]
    pub fn pair_mut(&mut self) -> &mut Pair<Simd<Bb, 4>> {
        unsafe { &mut self.pair }
    }

    #[inline]
    pub fn vec_mut(&mut self) -> &mut Simd<Bb, 8> {
        unsafe { &mut self.vec }
    }
}
