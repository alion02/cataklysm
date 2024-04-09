use crate::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub union Influence {
    pub pair: PovPair<Simd<Bb, 4>>,
    pub vec: Simd<Bb, 8>,
    pub arr: [Simd<Bb, 4>; 2],
}

impl Influence {
    pub const EMPTY: Simd<Bb, 8> = Simd::from_array([0; 8]);
    pub const EDGES: Simd<Bb, 8> = Simd::from_array([
        EDGE_LEFT,
        EDGE_BOTTOM,
        EDGE_RIGHT,
        EDGE_TOP,
        EDGE_LEFT,
        EDGE_BOTTOM,
        EDGE_RIGHT,
        EDGE_TOP,
    ]);

    #[inline]
    pub fn swap(&mut self) {
        let v = self.vec_mut();
        *v = simd_swizzle!(*v, [4, 5, 6, 7, 0, 1, 2, 3]);
    }

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
