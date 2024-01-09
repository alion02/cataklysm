use crate::{stack::*, util::*};

pub mod size3 {
    use super::*;

    pub const SIZE: usize = 3;
    pub const ROW_LEN: usize = 4;

    pub type Bitboard = u32;
    pub type Bits = Bits32<1>;
    pub type Stack = Stack32;
    pub type ActionBacking = u16;

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

pub mod size4 {
    use super::*;

    pub const SIZE: usize = 4;
    pub const ROW_LEN: usize = 5; // TODO

    pub type Bitboard = u32;
    pub type Bits = Bits32<1>;
    pub type Stack = Stack32;
    pub type ActionBacking = u16;

    pub const PERFT: &[(u32, u64)] = &[
        (1, 16),
        (2, 240),
        (3, 7440),
        (4, 216464),
        (5, 6468872),
        // (6, 181954216),
    ];
}

pub mod size5 {
    use super::*;

    pub const SIZE: usize = 5;
    pub const ROW_LEN: usize = 6;

    pub type Bitboard = u32;
    pub type Bits = Bits32<1>;
    pub type Stack = Stack64;
    pub type ActionBacking = u16;

    pub const PERFT: &[(u32, u64)] = &[
        (1, 25),
        (2, 600),
        (3, 43320),
        (4, 2999784),
        (5, 187855252),
        //
    ];
}

pub mod size6 {
    use super::*;

    pub const SIZE: usize = 6;
    pub const ROW_LEN: usize = 8;

    pub type Bitboard = u64;
    pub type Bits = Bits64<1>;
    pub type Stack = Stack64;
    pub type ActionBacking = u16;

    pub const PERFT: &[(u32, u64)] = &[
        (1, 36),
        (2, 1260),
        (3, 132720),
        (4, 13586048),
        // (5, 1253506520),
        // (6, 112449385016),
    ];
}

pub mod size7 {
    use super::*;

    pub const SIZE: usize = 7;
    pub const ROW_LEN: usize = 8;

    pub type Bitboard = u64;
    pub type Bits = Bits64<1>;
    pub type Stack = Stack128;
    pub type ActionBacking = u16;

    pub const PERFT: &[(u32, u64)] = &[
        (1, 49),
        (2, 2352),
        (3, 339696),
        (4, 48051008),
        //
    ];
}

pub mod size8 {
    use super::*;

    pub const SIZE: usize = 8;
    pub const ROW_LEN: usize = 9; // TODO

    pub type Bitboard = u128;
    pub type Bits = Bits128<1>;
    pub type Stack = Stack128;
    pub type ActionBacking = u32;

    pub const PERFT: &[(u32, u64)] = &[
        (1, 64),
        (2, 4032),
        (3, 764064),
        (4, 142512336),
        //
    ];
}
