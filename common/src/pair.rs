use core::{
    mem::transmute,
    ops::{Index, IndexMut},
};

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

impl<T> Index<bool> for Pair<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: bool) -> &T {
        unsafe { &transmute::<_, &[T; 2]>(self)[index as usize] }
    }
}

impl<T> IndexMut<bool> for Pair<T> {
    #[inline]
    fn index_mut(&mut self, index: bool) -> &mut T {
        unsafe { &mut transmute::<_, &mut [T; 2]>(self)[index as usize] }
    }
}
