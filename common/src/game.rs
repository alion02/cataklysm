use alloc::{boxed::Box, sync::Arc};
use core::{
    any::Any,
    fmt,
    ops::{Add, Neg, Sub},
    sync::atomic::{AtomicBool, Ordering::Relaxed},
};

use crate::{
    hash::Hash,
    pair::Pair,
    params::{SearchParams, SEARCH_PARAMS},
};

pub struct Options {
    pub start_stones: Pair<u32>,
    pub start_caps: Pair<u32>,
    pub half_komi: i32,

    pub params: SearchParams,
}

impl Options {
    pub fn default(size: usize) -> Option<Self> {
        let (stones, caps) = match size {
            3 => (10, 0),
            4 => (15, 0),
            5 => (21, 1),
            6 => (30, 1),
            7 => (40, 2),
            8 => (50, 2),
            _ => return None,
        };

        Some(Self {
            start_stones: Pair::both(stones),
            start_caps: Pair::both(caps),
            half_komi: 0,

            params: SEARCH_PARAMS,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PerftMode {
    Naive,
    Batch,
}

pub trait Move: fmt::Display + Send {
    fn as_any(self: Box<Self>) -> Box<dyn Any>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Eval(i32);

impl fmt::Display for Eval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_decisive() {
            let move_number = (Self::MAX.0 - self.0.abs() + 1) / 2;
            if *self > Self::ZERO {
                write!(f, "win on {move_number}")
            } else {
                write!(f, "loss on {move_number}")
            }
        } else {
            write!(f, "{:+}", self.0)
        }
    }
}

impl Neg for Eval {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Add<i32> for Eval {
    type Output = Self;

    #[inline]
    fn add(self, rhs: i32) -> Self {
        Self::new(self.0 + rhs)
    }
}

impl Sub<i32> for Eval {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: i32) -> Self {
        Self::new(self.0 - rhs)
    }
}

impl Eval {
    pub const ZERO: Self = Self(0);
    pub const DECISIVE: Self = Self(1 << 16);
    pub const MAX: Self = Self(1 << 24);

    #[inline]
    pub fn new(value: i32) -> Self {
        let eval = Self(value);
        debug_assert!(eval.abs() <= Self::MAX);
        eval
    }

    #[inline]
    pub fn win(ply: u32) -> Eval {
        Self::new(Self::MAX.0 - ply as i32)
    }

    #[inline]
    pub fn loss(ply: u32) -> Eval {
        Self::new(ply as i32 - Self::MAX.0)
    }

    #[inline]
    pub fn is_decisive(self) -> bool {
        self.abs() >= Self::DECISIVE
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    #[inline]
    pub fn raw(self) -> i32 {
        self.0
    }
}

#[derive(Debug)]
pub struct ParseMoveError;

#[derive(Debug)]
pub struct PlayMoveError;

#[derive(Debug)]
pub struct SetPositionError;

pub struct AbortFlag(Arc<AtomicBool>);

impl AbortFlag {
    #[inline]
    pub fn new(flag: &Arc<AtomicBool>) -> Self {
        Self(flag.clone())
    }

    #[inline]
    pub fn set(self) {
        self.0.store(true, Relaxed);
    }
}

pub trait Game: Send {
    fn search(&mut self, depth: u32) -> Option<(Eval, Box<dyn Move>)>;
    fn perft(&mut self, depth: u32, mode: PerftMode) -> u64;
    fn parser(&mut self) -> fn(&str) -> Result<Box<dyn Move>, ParseMoveError>;
    fn play(&mut self, mv: Box<dyn Move>) -> Result<(), PlayMoveError>;
    fn set_position(&mut self, tps: &str) -> Result<(), SetPositionError>;
    fn pv(&mut self) -> Box<dyn fmt::Display + '_>;
    fn abort_flag(&self) -> AbortFlag;
    fn clear_abort_flag(&self) -> bool;
    fn swap_abort_flags(&mut self);
    fn nodes(&self) -> u64;
    fn clear_nodes(&mut self);
    fn hash(&mut self) -> Hash;
    fn stones_left(&self) -> Pair<u32>;
    fn caps_left(&self) -> Pair<u32>;
    fn active_color(&self) -> bool;
    fn is_opening(&self) -> bool;
}

#[derive(Debug)]
pub struct NewGameError;

impl fmt::Display for NewGameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error instantiating a game")
    }
}

pub fn size_of_tps(tps: &str) -> usize {
    tps.as_bytes().iter().filter(|&&c| c == b'/').count() + 1
}
