use core::{
    mem::transmute,
    ops::{Index, IndexMut},
};

use crate::Color;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pair<T> {
    pub white: T,
    pub black: T,
}

impl<T> Pair<T> {
    #[inline]
    pub fn new(white: T, black: T) -> Self {
        Self { white, black }
    }
}

impl<T: Copy> Pair<T> {
    #[inline]
    pub fn both(value: T) -> Self {
        Self {
            white: value,
            black: value,
        }
    }
}

impl<T> Index<Color> for Pair<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: Color) -> &T {
        unsafe { &transmute::<_, &[T; 2]>(self)[index as usize] }
    }
}

impl<T> IndexMut<Color> for Pair<T> {
    #[inline]
    fn index_mut(&mut self, index: Color) -> &mut T {
        unsafe { &mut transmute::<_, &mut [T; 2]>(self)[index as usize] }
    }
}
