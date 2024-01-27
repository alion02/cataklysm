use std::{
    any::Any,
    error::Error,
    fmt,
    ops::Neg,
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc,
    },
};

use crate::{hash::Hash, pair::Pair, state::*};

pub struct Options {
    pub start_stones: Pair<u32>,
    pub start_caps: Pair<u32>,
    pub half_komi: i32,
    pub tt_size: usize,
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
            tt_size: 1 << 24,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PerftMode {
    Naive,
    Batch,
}

pub trait Action: fmt::Display + Send {
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

    #[inline(always)]
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Eval {
    pub const ZERO: Self = Self(0);
    pub const DECISIVE: Self = Self(1 << 16);
    pub const MAX: Self = Self(1 << 24);

    #[inline(always)]
    pub fn new(value: i32) -> Self {
        let eval = Self(value);
        debug_assert!(eval.abs() <= Self::MAX);
        eval
    }

    #[inline(always)]
    pub fn win(ply: u32) -> Eval {
        Self::new(Self::MAX.0 - ply as i32)
    }

    #[inline(always)]
    pub fn loss(ply: u32) -> Eval {
        Self::new(ply as i32 - Self::MAX.0)
    }

    #[inline(always)]
    pub fn is_decisive(self) -> bool {
        self.abs() >= Self::DECISIVE
    }

    #[inline(always)]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    #[inline(always)]
    pub fn raw(self) -> i32 {
        self.0
    }
}

#[derive(Debug)]
pub struct ParseActionError;

#[derive(Debug)]
pub struct PlayActionError;

#[derive(Debug)]
pub struct SetPositionError;

pub struct AbortFlag(Arc<AtomicBool>);

impl AbortFlag {
    pub fn new(flag: &Arc<AtomicBool>) -> Self {
        Self(flag.clone())
    }

    pub fn set(self) {
        self.0.store(true, Relaxed);
    }
}

pub trait Game: Send {
    fn perft(&mut self, depth: u32, mode: PerftMode) -> u64;
    fn search(&mut self, depth: u32) -> (Eval, Box<dyn Action>);
    fn parse_action(&mut self, ptn: &str) -> Result<Box<dyn Action>, ParseActionError>;
    fn play(&mut self, action: Box<dyn Action>) -> Result<(), PlayActionError>;
    fn set_position(&mut self, tps: &str) -> Result<(), SetPositionError>;
    fn take_nodes(&mut self) -> u64;
    fn curr_hash(&mut self) -> Hash;
    fn abort_flag(&mut self) -> AbortFlag;
    fn clear_abort_flag(&mut self) -> bool;
    fn stones_left(&mut self) -> Pair<u32>;
    fn caps_left(&mut self) -> Pair<u32>;
    fn active_color(&mut self) -> bool;
}

#[derive(Debug)]
pub struct NewGameError;

impl fmt::Display for NewGameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error instantiating a game")
    }
}

impl Error for NewGameError {}

pub fn new_game(size: usize, opt: Options) -> Result<Box<dyn Game>, NewGameError> {
    Ok(match size {
        #[cfg(feature = "3")]
        3 => Box::new(State3::new(opt)?),
        #[cfg(feature = "4")]
        4 => Box::new(State4::new(opt)?),
        #[cfg(feature = "5")]
        5 => Box::new(State5::new(opt)?),
        #[cfg(feature = "6")]
        6 => Box::new(State6::new(opt)?),
        #[cfg(feature = "7")]
        7 => Box::new(State7::new(opt)?),
        #[cfg(feature = "8")]
        8 => Box::new(State8::new(opt)?),
        _ => return Err(NewGameError),
    })
}

pub fn size_of_tps(tps: &str) -> usize {
    tps.as_bytes().iter().filter(|&&c| c == b'/').count() + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(1, 72)]
    #[case(2, 4655)]
    #[case(3, 332432)]
    #[case(4, 21315929)]
    #[cfg_attr(not(debug_assertions), case(5, 1506310007))]
    fn perft_early(#[case] depth: u32, #[case] expected: u64) {
        let mut game = new_game(6, Options::default(6).unwrap()).unwrap();
        game.set_position("x4,2C,1/x4,1C,x/x2,1S,1,121,x/x,2,x4/x3,2S,2S,x/2,x5 1 8")
            .unwrap();
        assert_eq!(game.perft(depth, PerftMode::Batch), expected);
    }
}
