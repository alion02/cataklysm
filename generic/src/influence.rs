use crate::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub union Influence {
    pub pair: PovPair<Simd<Bb, 4>>,
    pub vec: Simd<Bb, 8>,
    pub arr: [Simd<Bb, 4>; 2],
}

impl Influence {
    #[inline]
    pub fn pair_mut(&mut self) -> &mut PovPair<Simd<Bb, 4>> {
        unsafe { &mut self.pair }
    }

    #[inline]
    pub fn vec_mut(&mut self) -> &mut Simd<Bb, 8> {
        unsafe { &mut self.vec }
    }

    #[inline]
    pub fn arr_mut(&mut self) -> &mut [Simd<Bb, 4>; 2] {
        unsafe { &mut self.arr }
    }
}
