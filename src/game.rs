use std::{error::Error, fmt, ops::Neg};

use crate::{pair::Pair, state::*};

pub enum Position<'a> {
    Start(usize),
    Tps(&'a str),
}

impl<'a> Position<'a> {
    pub fn size(&self) -> usize {
        match self {
            Self::Start(s) => *s,
            Self::Tps(t) => t.as_bytes().iter().filter(|&&c| c == b'/').count() + 1,
        }
    }
}

pub struct Options<'a> {
    pub position: Position<'a>,
    pub start_stones: Pair<u32>,
    pub start_caps: Pair<u32>,
    pub half_komi: i32,
}

impl<'a> Options<'a> {
    pub fn from_position(position: Position<'a>) -> Option<Self> {
        let (stones, caps) = match position.size() {
            3 => (10, 0),
            4 => (15, 0),
            5 => (21, 1),
            6 => (30, 1),
            7 => (40, 2),
            8 => (50, 2),
            _ => return None,
        };

        Some(Self {
            position,
            start_stones: Pair::both(stones),
            start_caps: Pair::both(caps),
            half_komi: 0,
        })
    }

    pub fn from_tps(tps: &'a str) -> Option<Self> {
        Self::from_position(Position::Tps(tps))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PerftMode {
    Naive,
    Batch,
}

pub trait Action: fmt::Display {}

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
}

pub trait Game {
    fn perft(&mut self, depth: u32, mode: PerftMode) -> u64;
    fn search(&mut self, depth: u32) -> (Eval, Option<Box<dyn Action>>);
}

#[derive(Debug)]
pub struct NewGameError;

impl fmt::Display for NewGameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error instantiating a game")
    }
}
impl Error for NewGameError {}

pub fn new_game(opt: Options) -> Result<Box<dyn Game>, NewGameError> {
    Ok(match opt.position.size() {
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
        assert_eq!(
            new_game(
                Options::from_tps("x4,2C,1/x4,1C,x/x2,1S,1,121,x/x,2,x4/x3,2S,2S,x/2,x5 1 8")
                    .unwrap()
            )
            .unwrap()
            .perft(depth, PerftMode::Batch),
            expected
        );
    }
}
