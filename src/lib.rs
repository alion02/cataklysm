use common::game::*;
pub use common::*;

pub fn new_game(size: usize, opt: Options) -> Result<Box<dyn Game>, NewGameError> {
    Ok(match size {
        #[cfg(feature = "3")]
        3 => Box::new(size3::State::new(opt)?),
        #[cfg(feature = "4")]
        4 => Box::new(size4::State::new(opt)?),
        #[cfg(feature = "5")]
        5 => Box::new(size5::State::new(opt)?),
        #[cfg(feature = "6")]
        6 => Box::new(size6::State::new(opt)?),
        #[cfg(feature = "7")]
        7 => Box::new(size7::State::new(opt)?),
        #[cfg(feature = "8")]
        8 => Box::new(size8::State::new(opt)?),
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
        let mut game = new_game(6, Options::default(6).unwrap()).unwrap();
        game.set_position("x4,2C,1/x4,1C,x/x2,1S,1,121,x/x,2,x4/x3,2S,2S,x/2,x5 1 8")
            .unwrap();
        assert_eq!(game.perft(depth, PerftMode::Batch), expected);
    }
}
