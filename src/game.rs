use std::{error::Error, fmt::Display};

use crate::{pair::Pair, state::*};

pub enum Position<'a> {
    Start(usize),
    Tps(&'a str),
}

pub struct Options<'a> {
    pub position: Position<'a>,
    pub start_stones: Pair<u32>,
    pub start_caps: Pair<u32>,
    pub half_komi: i32,
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
    let size = match opt.position {
        Position::Start(s) => s,
        Position::Tps(t) => t.as_bytes().iter().filter(|&&c| c == b'/').count() + 1,
    };

    Ok(Box::new(match size {
        6 => State6::new(opt)?,
        _ => return Err(NewGameError),
    }))
}
