#![allow(dead_code)]

use std::{
    fmt::{self, Display},
    mem::transmute,
    ops::{
        ControlFlow::{self, *},
        Index, IndexMut,
    },
};

use crate::{game::*, pair::*, stack::*, util::*};
use Direction::*;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Piece {
    Flat = 0b01,
    Wall = 0b10,
    Cap = 0b11,
}

impl Piece {
    #[inline(always)]
    fn is_road(self) -> bool {
        self as u32 & 1 != 0
    }

    #[inline(always)]
    fn is_block(self) -> bool {
        self as u32 & 2 != 0
    }

    #[inline(always)]
    fn is_stone(self) -> bool {
        self != Self::Cap
    }
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Right,
    Up,
    Left,
    Down,
}

macro_rules! state {
    ($mod:ident => $export:ident {
        size: $SIZE:expr,
        row_len: $ROW_LEN:expr,
        bitboard: $Bitboard:ident,
        stack: $Stack:ident,
        action: $ActionBacking:ident,
        perft: {$($depth:expr => $expected_perft:expr,)*},
    }) => {
        pub(crate) use $mod::State as $export;
        mod $mod {
            use super::*;

            // size-dependent parameters

            type Bitboard = $Bitboard;
            type Stack = $Stack;
            type ActionBacking = $ActionBacking;

            const SIZE: usize = $SIZE;
            const ROW_LEN: usize = $ROW_LEN;

            // end of parameters

            const HAND: u32 = SIZE as u32;

            const PADDING: usize = ROW_LEN - SIZE;
            const ARR_LEN: usize = SIZE * ROW_LEN - PADDING;

            const ROW: Bitboard = (1 << SIZE) - 1;
            const COL: Bitboard = {
                let mut col: Bitboard = 1;
                while col.count_ones() < SIZE as u32 {
                    col |= col << ROW_LEN;
                }
                col
            };

            const BOARD: Bitboard = ROW * COL;

            fn row(sq: Square) -> Bitboard {
                ROW << sq.row().0
            }

            fn col(sq: Square) -> Bitboard {
                COL << sq.col().0
            }

            fn ray(src: Square, dir: Direction) -> Bitboard {
                match dir {
                    Right => row(src) & !1 << src.0,
                    Up => col(src) & !1 << src.0,
                    Left => row(src) & (1 << src.0) - 1,
                    Down => col(src) & (1 << src.0) - 1,
                }
            }

            fn closest_hit(ray_hits: Bitboard, dir: Direction) -> Bitboard {
                ray_hits
                    & match dir {
                        Right | Up => ray_hits.wrapping_neg(),
                        Left | Down => {
                            !(!(0 as Bitboard) >> 1).wrapping_shr(ray_hits.leading_zeros())
                        }
                    }
            }

            fn distance(src: Square, hit: Square, dir: Direction) -> u32 {
                (match dir {
                    Right => hit.0 - src.0,
                    Up => (hit.0 - src.0) / ROW_LEN,
                    Left => src.0 - hit.0,
                    Down => (src.0 - hit.0) / ROW_LEN,
                }) as u32
            }

            #[derive(Clone, Copy, PartialEq, Eq)]
            struct Square(usize);

            impl<T> Index<Square> for [T] {
                type Output = T;

                fn index(&self, index: Square) -> &Self::Output {
                    unsafe { self.get(index.0).unwrap_unchecked() }
                }
            }

            impl<T> IndexMut<Square> for [T] {
                fn index_mut(&mut self, index: Square) -> &mut Self::Output {
                    unsafe { self.get_mut(index.0).unwrap_unchecked() }
                }
            }

            impl Square {
                fn bit(self) -> Bitboard {
                    1 << self.0
                }

                #[must_use]
                fn shift(self, dir: Direction) -> Self {
                    sq(match dir {
                        Right => self.0 + 1,
                        Up => self.0 + ROW_LEN,
                        Left => self.0 - 1,
                        Down => self.0 - ROW_LEN,
                    })
                }

                #[must_use]
                fn row(self) -> Self {
                    sq(self.0 / ROW_LEN * ROW_LEN)
                }

                #[must_use]
                fn col(self) -> Self {
                    sq(self.0 % ROW_LEN)
                }
            }

            fn sq(sq: usize) -> Square {
                debug_assert!(sq < ARR_LEN);
                debug_assert!(sq % ROW_LEN < SIZE);

                Square(sq)
            }

            #[derive(Clone, Copy, PartialEq, Eq)]
            struct Pattern(u32);

            impl Pattern {
                fn execute(self) -> (u32, DropCounts) {
                    let mut dc = DropCounts(self.0 | 1 << HAND);
                    (HAND - dc.next().unwrap(), dc)
                }
            }

            fn pat(pat: u32) -> Pattern {
                debug_assert!(pat > 0);
                debug_assert!(pat < 1 << HAND);

                Pattern(pat)
            }

            #[derive(Clone, Copy, PartialEq, Eq)]
            struct Action(ActionBacking);

            impl Display for Action {
                fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                    todo!()
                }
            }

            impl Action {
                const TYPE_OFFSET: u32 = (ARR_LEN - 1).ilog2() + 1;
                const PAT_OFFSET: u32 = Self::TYPE_OFFSET + 2;

                fn pass() -> Self {
                    Self(0)
                }

                fn place(sq: Square, piece: Piece) -> Self {
                    Self(sq.0 as ActionBacking | (piece as ActionBacking) << Self::TYPE_OFFSET)
                }

                fn spread(sq: Square, dir: Direction, pat: Pattern) -> Self {
                    Self(
                        sq.0 as ActionBacking
                            | (dir as ActionBacking) << Self::TYPE_OFFSET
                            | (pat.0 as ActionBacking) << Self::PAT_OFFSET,
                    )
                }

                fn branch<S, R>(
                    self,
                    state: S,
                    pass: impl FnOnce(S) -> R,
                    place: impl FnOnce(S, Square, Piece) -> R,
                    spread: impl FnOnce(S, Square, Direction, Pattern) -> R,
                ) -> R {
                    if self.0 == 0 {
                        pass(state)
                    } else {
                        let sq = sq(self.0 as usize & (1 << Self::TYPE_OFFSET) - 1);
                        if self.0 < 1 << Self::PAT_OFFSET {
                            place(state, sq, unsafe {
                                transmute(self.0 as u32 >> Self::TYPE_OFFSET)
                            })
                        } else {
                            spread(
                                state,
                                sq,
                                unsafe { transmute(self.0 as u32 >> Self::TYPE_OFFSET & 3) },
                                pat(self.0 as u32 >> Self::PAT_OFFSET),
                            )
                        }
                    }
                }
            }

            #[repr(C)]
            pub struct State {
                road: Pair<Bitboard>,
                block: Pair<Bitboard>,

                stones_left: Pair<u32>,
                caps_left: Pair<u32>,

                ply: u32,

                stacks: [Stack; ARR_LEN],
            }

            impl Default for State {
                fn default() -> Self {
                    Self::new(Options::from_position(Position::Start(SIZE)).unwrap()).unwrap()
                }
            }

            impl State {
                pub(crate) fn new(opt: Options) -> Result<Self, NewGameError> {
                    let mut road = Pair::both(0);
                    let mut block = Pair::both(0);
                    let mut stones_left = opt.start_stones;
                    let mut caps_left = opt.start_caps;
                    let mut ply = 0;
                    let mut stacks = [Stack::EMPTY; ARR_LEN];

                    match opt.position {
                        Position::Start(s) => {
                            if s != SIZE {
                                return Err(NewGameError);
                            }
                        }
                        Position::Tps(s) => {
                            use takparse::{Color, Piece, Tps};

                            let tps: Tps = s.parse().map_err(|_| NewGameError)?;
                            for (row, y) in tps.board_2d().zip((0..SIZE).rev()) {
                                for (stack, x) in row.zip(0..SIZE) {
                                    if let Some(stack) = stack {
                                        let sq = sq(x + y * ROW_LEN);

                                        for color in stack.colors() {
                                            let color = color != Color::White;
                                            stacks[sq].drop(&mut Hand::one_piece(color), 1);
                                            stones_left[color] -= 1;
                                        }

                                        let top = stack.top();
                                        let color = stack.top_color() != Color::White;

                                        if matches!(top, Piece::Flat | Piece::Cap) {
                                            road[color] |= sq.bit();
                                        }
                                        if matches!(top, Piece::Wall | Piece::Cap) {
                                            block[color] |= sq.bit();
                                        }

                                        if top == Piece::Cap {
                                            stones_left[color] += 1; // Correct overcounting from the stack
                                            caps_left[color] -= 1;
                                        }
                                    }
                                }
                            }

                            ply = tps.ply() as u32;
                        }
                    }

                    Ok(Self {
                        road,
                        block,
                        stones_left,
                        caps_left,
                        ply,
                        stacks,
                    })
                }

                fn color(&self) -> bool {
                    self.ply & 1 != 0
                }

                fn is_opening(&self) -> bool {
                    self.ply < 2
                }

                fn for_actions<B, C>(
                    &mut self,
                    mut acc: C,
                    mut f: impl FnMut(C, &mut Self, Action) -> ControlFlow<B, C>,
                ) -> ControlFlow<B, C> {
                    let color = self.color();

                    let own = self.road[color] | self.block[color];
                    let empty = BOARD ^ own ^ (self.road[!color] | self.block[!color]);

                    let block = self.block.white | self.block.black;
                    let cap = block & (self.road.white | self.road.black);

                    let has_stones = self.stones_left[color] > 0;
                    let has_caps = self.caps_left[color] > 0;
                    let is_opening = self.is_opening();

                    let mut remaining = empty;
                    loop {
                        let sq = sq(remaining.trailing_zeros() as usize);

                        'skip_nobles: {
                            if has_stones {
                                acc = f(acc, self, Action::place(sq, Piece::Flat))?;

                                if is_opening {
                                    break 'skip_nobles;
                                }

                                acc = f(acc, self, Action::place(sq, Piece::Wall))?;
                            }

                            if has_caps {
                                acc = f(acc, self, Action::place(sq, Piece::Cap))?;
                            }
                        }

                        remaining &= remaining - 1;
                        if remaining == 0 {
                            break;
                        }
                    }

                    if !is_opening {
                        remaining = own;
                        while remaining != 0 {
                            let src = sq(remaining.trailing_zeros() as usize);
                            let src_bit = remaining & remaining.wrapping_neg();

                            let is_cap = src_bit & cap != 0;

                            let max_pieces = self.stacks[src].height().min(HAND);
                            let start_bit = 1 << HAND >> max_pieces;

                            // TODO PERF: Removing the assert significantly reduces performance
                            assert_ne!(max_pieces, 0);

                            let mut spread = {
                                #[inline(always)]
                                |mut acc, dir| {
                                    let ray = ray(src, dir);
                                    let ray_hits = ray & block;
                                    let ray_hit = closest_hit(ray_hits, dir);

                                    let range = if ray_hit != 0 {
                                        distance(src, sq(ray_hit.trailing_zeros() as usize), dir)
                                            - 1
                                    } else {
                                        ray.count_ones()
                                    };

                                    let mut do_spreads = |mut acc, mut pattern, range, limit| {
                                        if range > 0 {
                                            while pattern < limit {
                                                acc = f(
                                                    acc,
                                                    self,
                                                    Action::spread(src, dir, pat(pattern)),
                                                )?;

                                                pattern += if pattern.count_ones() == range {
                                                    pattern & pattern.wrapping_neg()
                                                } else {
                                                    start_bit
                                                };
                                            }
                                        }

                                        Continue(acc)
                                    };

                                    if is_cap && ray_hit & !cap != 0 {
                                        // Smash possible
                                        acc = do_spreads(acc, start_bit, range, 1 << HAND - 1)?;
                                        acc = do_spreads(acc, 1 << HAND - 1, range + 1, 1 << HAND)?;
                                    } else {
                                        acc = do_spreads(acc, start_bit, range, 1 << HAND)?;
                                    }

                                    Continue(acc)
                                }
                            };

                            acc = spread(acc, Right)?;
                            acc = spread(acc, Up)?;
                            acc = spread(acc, Left)?;
                            acc = spread(acc, Down)?;

                            remaining &= remaining - 1;
                        }
                    }

                    Continue(acc)
                }

                fn with<R>(
                    &mut self,
                    undo: bool,
                    action: Action,
                    f: impl FnOnce(&mut Self) -> R,
                ) -> R {
                    let mut s = self;
                    let color = s.color() ^ s.is_opening();

                    debug_assert!(s.is_legal(action));

                    s.ply += 1;

                    let r = action.branch(
                        (&mut s, f),
                        |(s, f)| f(s),
                        |(s, f), sq, piece| {
                            let bit = sq.bit();

                            if piece.is_road() {
                                s.road[color] ^= bit;
                            }

                            if piece.is_block() {
                                s.block[color] ^= bit;
                            }

                            if piece.is_stone() {
                                s.stones_left[color] -= 1;
                            } else {
                                s.caps_left[color] -= 1;
                            }

                            s.stacks[sq] = Stack::one_tall(color);

                            let r = f(s);

                            if undo {
                                s.stacks[sq] = Stack::EMPTY;

                                if piece.is_stone() {
                                    s.stones_left[color] += 1;
                                } else {
                                    s.caps_left[color] += 1;
                                }

                                if piece.is_block() {
                                    s.block[color] ^= bit;
                                }

                                if piece.is_road() {
                                    s.road[color] ^= bit;
                                }
                            }

                            r
                        },
                        |(s, f), mut sq, dir, pat| {
                            let mut bit = sq.bit();

                            let road = s.road;
                            let block = s.block;
                            let stacks = s.stacks;

                            let is_road = road[color] & bit != 0;
                            let is_block = block[color] & bit != 0;

                            let (taken, counts) = pat.execute();

                            let mut hand = s.stacks[sq].take(taken);
                            let top = s.stacks[sq].top();

                            if top.is_none() | top.is_some_and(|new_color| new_color != color) {
                                s.road[color] &= !bit;
                            }
                            if let Some(new_color) = top {
                                s.road[new_color] |= bit;
                            }
                            s.block[color] &= !bit;

                            for count in counts {
                                sq = sq.shift(dir);
                                bit = sq.bit();

                                s.stacks[sq].drop(&mut hand, count);

                                s.road.white &= !bit;
                                s.road.black &= !bit;

                                s.road[s.stacks[sq].top_unchecked()] |= bit;
                            }

                            if is_block {
                                if is_road {
                                    s.block[!color] &= !bit;
                                } else {
                                    s.road[color] &= !bit;
                                }
                                s.block[color] |= bit;
                            }

                            let r = f(s);

                            if undo {
                                s.stacks = stacks;
                                s.block = block;
                                s.road = road;
                            }

                            r
                        },
                    );

                    if undo {
                        s.ply -= 1;
                    }

                    r
                }

                fn status<S, R>(
                    &mut self,
                    state: S,
                    ongoing: impl FnOnce(S, &mut Self) -> R,
                    draw: impl FnOnce(S, &mut Self) -> R,
                    win: impl FnOnce(S, &mut Self) -> R,
                    loss: impl FnOnce(S, &mut Self) -> R,
                ) -> R {
                    let color = self.color();

                    if self.has_road(!color) {
                        return loss(state, self);
                    }

                    if self.has_road(color) {
                        return win(state, self);
                    }

                    if self.stones_left[!color] == 0 && self.caps_left[!color] == 0
                        || self.road.white | self.road.black | self.block.white | self.block.black
                            == BOARD
                    {
                        let own = self.count_flats(color);
                        let opp = self.count_flats(!color);

                        return if opp > own {
                            loss(state, self)
                        } else if own > opp {
                            win(state, self)
                        } else {
                            draw(state, self)
                        };
                    }

                    ongoing(state, self)
                }

                fn has_road(&self, color: bool) -> bool {
                    assert_ne!(PADDING, 0);

                    let road = self.road[color];

                    const BOTTOM: usize = 0;
                    const TOP: usize = 1;
                    const LEFT: usize = 2;
                    const RIGHT: usize = 3;

                    let mut edges = [0; 4];

                    edges[BOTTOM] = row(sq(0));
                    edges[TOP] = row(sq((SIZE - 1) * ROW_LEN));
                    edges[LEFT] = col(sq(0));
                    edges[RIGHT] = col(sq(SIZE - 1));

                    let mut curr = edges.map(|e| e & road);

                    loop {
                        // Fill all nearby road tiles
                        let next = curr
                            .map(|c| (c | c << 1 | c >> 1 | c << ROW_LEN | c >> ROW_LEN) & road);

                        if (next[BOTTOM] & next[TOP] != 0) | (next[LEFT] & next[RIGHT] != 0) {
                            // If either pair of edges met, there is a road
                            return true;
                        }

                        if ((next[BOTTOM] == curr[BOTTOM]) | (next[TOP] == curr[TOP]))
                            & ((next[LEFT] == curr[LEFT]) | (next[RIGHT] == curr[RIGHT]))
                        {
                            // If at least one edge stagnated in both directions, there can be no road
                            return false;
                        }

                        curr = next;
                    }
                }

                fn count_flats(&self, color: bool) -> u32 {
                    (self.road[color] & !self.block[color]).count_ones()
                }

                fn is_legal(&self, action: Action) -> bool {
                    let color = self.color();
                    action.branch(
                        (),
                        |_| false,
                        |_, sq, piece| {
                            self.stacks[sq].is_empty()
                                && if piece.is_stone() {
                                    self.stones_left[color] != 0
                                } else {
                                    self.caps_left[color] != 0
                                }
                        },
                        |_, sq, dir, pat| {
                            let (taken, counts) = pat.execute();
                            self.stacks[sq].height() >= taken
                            // TODO
                        },
                    )
                }
            }

            impl Game for State {
                fn perft(&mut self, depth: u32, mode: PerftMode) -> u64 {
                    match depth {
                        0 => 1,
                        1 if mode == PerftMode::Batch => self.status(
                            (),
                            |_, s| {
                                s.for_actions(0, |sum, _, _| Continue(sum + 1))
                                    .into_continue()
                            },
                            |_, _| 1,
                            |_, _| 1,
                            |_, _| 1,
                        ),
                        _ => self.status(
                            (),
                            |_, s| {
                                s.for_actions(0, |sum, s, action| {
                                    Continue(
                                        sum + s.with(true, action, |s| s.perft(depth - 1, mode)),
                                    )
                                })
                                .into_continue()
                            },
                            |_, _| 1,
                            |_, _| 1,
                            |_, _| 1,
                        ),
                    }
                }

                fn tinue(&mut self, depth: u32) -> Tinue<Box<dyn crate::game::Action>> {
                    Tinue::Unknown
                }
            }

            #[cfg(test)]
            mod tests {
                use super::*;
                use rstest::rstest;

                #[rstest]
                $(#[case($depth, $expected_perft)])*
                fn perft(#[case] depth: u32, #[case] expected: u64) {
                    assert_eq!(State::default().perft(depth, PerftMode::Batch), expected);
                }
            }
        }
    };
}

state!(size3 => State3 {
    size: 3,
    row_len: 4,
    bitboard: u16,
    stack: Stack32,
    action: u16,
    perft: {
        1 => 9,
        2 => 72,
        3 => 1200,
        4 => 17792,
        5 => 271812,
        6 => 3715592,
        // 7 => 52400728,
    },
});

state!(size4 => State4 {
    size: 4,
    row_len: 5, // TODO
    bitboard: u32,
    stack: Stack32,
    action: u16,
    perft: {
        1 => 16,
        2 => 240,
        3 => 7440,
        4 => 216464,
        5 => 6468872,
        // 6 => 181954216,
    },
});

state!(size5 => State5 {
    size: 5,
    row_len: 6,
    bitboard: u32,
    stack: Stack64,
    action: u16,
    perft: {
        1 => 25,
        2 => 600,
        3 => 43320,
        4 => 2999784,
        5 => 187855252,
    },
});

state!(size6 => State6 {
    size: 6,
    row_len: 8,
    bitboard: u64,
    stack: Stack64,
    action: u16,
    perft: {
        1 => 36,
        2 => 1260,
        3 => 132720,
        4 => 13586048,
        // 5 => 1253506520,
        // 6 => 112449385016,
    },
});

state!(size7 => State7 {
    size: 7,
    row_len: 8,
    bitboard: u64,
    stack: Stack128,
    action: u16,
    perft: {
        1 => 49,
        2 => 2352,
        3 => 339696,
        4 => 48051008,
    },
});

state!(size8 => State8 {
    size: 8,
    row_len: 9, // TODO
    bitboard: u128,
    stack: Stack128,
    action: u32,
    perft: {
        1 => 64,
        2 => 4032,
        3 => 764064,
        4 => 142512336,
    },
});
