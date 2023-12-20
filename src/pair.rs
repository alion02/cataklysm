use std::ops::{Index, IndexMut};

// DO NOT CHANGE
// Current assignment assumed throughout codebase
pub const WHITE: bool = false;
pub const BLACK: bool = true;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pair<T> {
    pub white: T,
    pub black: T,
}

impl<T> Index<bool> for Pair<T> {
    type Output = T;

    fn index(&self, index: bool) -> &T {
        match index {
            WHITE => &self.white,
            BLACK => &self.black,
        }
    }
}

impl<T> IndexMut<bool> for Pair<T> {
    fn index_mut(&mut self, index: bool) -> &mut T {
        match index {
            WHITE => &mut self.white,
            BLACK => &mut self.black,
        }
    }
}

impl<T> Pair<T> {
    #[inline(always)]
    pub fn new(white: T, black: T) -> Self {
        Self { white, black }
    }

    #[inline(always)]
    pub fn get(self, color: bool) -> (T, T) {
        match color {
            WHITE => (self.white, self.black),
            BLACK => (self.black, self.white),
        }
    }

    #[inline(always)]
    pub fn get_ref(&self, color: bool) -> (&T, &T) {
        match color {
            WHITE => (&self.white, &self.black),
            BLACK => (&self.black, &self.white),
        }
    }

    #[inline(always)]
    pub fn get_mut(&mut self, color: bool) -> (&mut T, &mut T) {
        match color {
            WHITE => (&mut self.white, &mut self.black),
            BLACK => (&mut self.black, &mut self.white),
        }
    }
}

impl<T: Copy> Pair<T> {
    #[inline(always)]
    pub fn both(value: T) -> Self {
        Self {
            white: value,
            black: value,
        }
    }
}
