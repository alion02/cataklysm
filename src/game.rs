use std::{error::Error, fmt::Display};

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
    Specialized,
}

pub trait Game {
    fn perft(&mut self, depth: u32, mode: PerftMode) -> u64;
}

#[derive(Debug)]
pub struct NewGameError;

impl Display for NewGameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error instantiating a game")
    }
}
impl Error for NewGameError {}

pub fn new_game(opt: Options) -> Result<Box<dyn Game>, NewGameError> {
    Ok(Box::new(match opt.position.size() {
        6 => State6::new(opt)?,
        _ => return Err(NewGameError),
    }))
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
