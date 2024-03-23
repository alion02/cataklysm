#[cfg(feature = "3")]
mod inner {
    pub const SIZE: usize = 3;
    pub const ROW_LEN: usize = 4;

    pub type Bb = u32;
    pub type Stack = u32;

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
    pub const SIZE: usize = 4;
    pub const ROW_LEN: usize = 4;

    pub type Bb = u32;
    pub type Stack = u32;

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
    pub const SIZE: usize = 5;
    pub const ROW_LEN: usize = 5;

    pub type Bb = u32;
    pub type Stack = u64;

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
    pub const SIZE: usize = 6;
    pub const ROW_LEN: usize = 8;

    pub type Bb = u64;
    pub type Stack = u64;

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
    pub const SIZE: usize = 7;
    pub const ROW_LEN: usize = 8;

    pub type Bb = u64;
    pub type Stack = u128;

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
    pub const SIZE: usize = 8;
    pub const ROW_LEN: usize = 8;

    pub type Bb = u64;
    pub type Stack = u128;

    #[cfg(test)]
    pub const PERFT: &[(u32, u64)] = &[
        (1, 64),
        (2, 4032),
        (3, 764064),
        (4, 142512336),
        //
    ];
}

pub use inner::*;
