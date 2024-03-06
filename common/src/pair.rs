use crate::*;

use core::ops::{Index, IndexMut};

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

    #[inline]
    pub fn get(self, color: Color) -> (T, T) {
        match color {
            White => (self.white, self.black),
            Black => (self.black, self.white),
        }
    }

    #[inline]
    pub fn get_ref(&self, color: Color) -> (&T, &T) {
        match color {
            White => (&self.white, &self.black),
            Black => (&self.black, &self.white),
        }
    }

    #[inline]
    pub fn get_mut(&mut self, color: Color) -> (&mut T, &mut T) {
        match color {
            White => (&mut self.white, &mut self.black),
            Black => (&mut self.black, &mut self.white),
        }
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
        match index {
            White => &self.white,
            Black => &self.black,
        }
    }
}

impl<T> IndexMut<Color> for Pair<T> {
    #[inline]
    fn index_mut(&mut self, index: Color) -> &mut T {
        match index {
            White => &mut self.white,
            Black => &mut self.black,
        }
    }
}
