#![expect(unexpected_cfgs)] // Due to compiling the same code as multiple crates

use crate::*;

pub use inner::*;
pub use provider::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvalParams {
    pub max_dist_offset: i32,

    pub flat_count: i32,
    pub stones_left: i32,
    pub caps_left: i32,
    pub total_dist: i32,
    pub smallest_dist: i32,
    pub side_to_move: i32,
}

pub static EVAL_PARAMS: EvalParams = EvalParams {
    max_dist_offset: -1,

    flat_count: 10,
    stones_left: -7,
    caps_left: -15,
    total_dist: -1,
    smallest_dist: -2,
    side_to_move: 21,
};

#[cfg(feature = "runtime-config")]
mod provider {
    use super::*;

    #[derive(Debug)]
    pub struct SearchParamsProvider(SearchParams);

    impl SearchParamsProvider {
        pub fn new(params: SearchParams) -> Option<Self> {
            Some(Self(params))
        }
    }

    impl Deref for SearchParamsProvider {
        type Target = SearchParams;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug)]
    pub struct EvalParamsProvider(EvalParams);

    impl EvalParamsProvider {
        pub fn new(params: EvalParams) -> Option<Self> {
            Some(Self(params))
        }
    }

    impl Deref for EvalParamsProvider {
        type Target = EvalParams;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

#[cfg(not(feature = "runtime-config"))]
mod provider {
    use super::*;

    #[derive(Debug)]
    pub struct SearchParamsProvider;

    impl SearchParamsProvider {
        pub fn new(params: SearchParams) -> Option<Self> {
            (params == SEARCH_PARAMS).then_some(Self)
        }
    }

    impl Deref for SearchParamsProvider {
        type Target = SearchParams;

        fn deref(&self) -> &Self::Target {
            &SEARCH_PARAMS
        }
    }

    #[derive(Debug)]
    pub struct EvalParamsProvider;

    impl EvalParamsProvider {
        pub fn new(params: EvalParams) -> Option<Self> {
            (params == EVAL_PARAMS).then_some(Self)
        }
    }

    impl Deref for EvalParamsProvider {
        type Target = EvalParams;

        fn deref(&self) -> &Self::Target {
            &EVAL_PARAMS
        }
    }
}

#[cfg(feature = "3")]
mod inner {
    use super::*;

    pub const SIZE: usize = 3;
    pub const ROW_LEN: usize = 4;

    pub type Bitboard = u32;
    pub type Bits = Bits32<1>;
    pub type Stack = Stack32;
    pub type ActionBacking = u16;

    #[cfg(test)]
    pub const PERFT: &[(u32, u64)] = &[
        (1, 9),
        (2, 72),
        (3, 1200),
        (4, 17792),
        (5, 271812),
        (6, 3715592),
        // (7, 52400728),
    ];
}

#[cfg(feature = "4")]
mod inner {
    use super::*;

    pub const SIZE: usize = 4;
    pub const ROW_LEN: usize = 5; // TODO

    pub type Bitboard = u32;
    pub type Bits = Bits32<1>;
    pub type Stack = Stack32;
    pub type ActionBacking = u16;

    #[cfg(test)]
    pub const PERFT: &[(u32, u64)] = &[
        (1, 16),
        (2, 240),
        (3, 7440),
        (4, 216464),
        (5, 6468872),
        // (6, 181954216),
    ];
}

#[cfg(feature = "5")]
mod inner {
    use super::*;

    pub const SIZE: usize = 5;
    pub const ROW_LEN: usize = 6;

    pub type Bitboard = u32;
    pub type Bits = Bits32<1>;
    pub type Stack = Stack64;
    pub type ActionBacking = u16;

    #[cfg(test)]
    pub const PERFT: &[(u32, u64)] = &[
        (1, 25),
        (2, 600),
        (3, 43320),
        (4, 2999784),
        (5, 187855252),
        //
    ];
}

#[cfg(feature = "6")]
mod inner {
    use super::*;

    pub const SIZE: usize = 6;
    pub const ROW_LEN: usize = 8;

    pub type Bitboard = u64;
    pub type Bits = Bits64<1>;
    pub type Stack = Stack64;
    pub type ActionBacking = u16;

    #[cfg(test)]
    pub const PERFT: &[(u32, u64)] = &[
        (1, 36),
        (2, 1260),
        (3, 132720),
        (4, 13586048),
        // (5, 1253506520),
        // (6, 112449385016),
    ];
}

#[cfg(feature = "7")]
mod inner {
    use super::*;

    pub const SIZE: usize = 7;
    pub const ROW_LEN: usize = 8;

    pub type Bitboard = u64;
    pub type Bits = Bits64<1>;
    pub type Stack = Stack128;
    pub type ActionBacking = u16;

    #[cfg(test)]
    pub const PERFT: &[(u32, u64)] = &[
        (1, 49),
        (2, 2352),
        (3, 339696),
        (4, 48051008),
        //
    ];
}

#[cfg(feature = "8")]
mod inner {
    use super::*;

    pub const SIZE: usize = 8;
    pub const ROW_LEN: usize = 8;

    pub type Bitboard = u64;
    pub type Bits = Bits64<1>;
    pub type Stack = Stack128;
    pub type ActionBacking = u16;

    #[cfg(test)]
    pub const PERFT: &[(u32, u64)] = &[
        (1, 64),
        (2, 4032),
        (3, 764064),
        (4, 142512336),
        //
    ];
}
