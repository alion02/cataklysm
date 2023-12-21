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
}

pub trait Game {}

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
