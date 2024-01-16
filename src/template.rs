#![allow(clippy::items_after_test_module)]

use std::{
    fmt,
    mem::transmute,
    ops::{
        ControlFlow::{self, *},
        Index, IndexMut,
    },
    sync::Mutex,
};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

use crate::{game::*, hash::*, pair::*, stack::*, state::*, util::*};
use Direction::*;

static INIT: Mutex<bool> = Mutex::new(false);

static mut HASH_WALL: [Hash; ARR_LEN] = [Hash::ZERO; ARR_LEN];
static mut HASH_CAP: [Hash; ARR_LEN] = [Hash::ZERO; ARR_LEN];
static mut HASH_STACK: [[[Hash; 2 << HAND]; Stack::CAPACITY as usize]; ARR_LEN] =
    [[[Hash::ZERO; 2 << HAND]; Stack::CAPACITY as usize]; ARR_LEN];

fn init() {
    let mut init = INIT.lock().unwrap();
    if !*init {
        let mut rng = ChaCha20Rng::seed_from_u64(0);

        unsafe {
            HASH_WALL.iter_mut().for_each(|h| *h = rng.gen());
            HASH_CAP.iter_mut().for_each(|h| *h = rng.gen());

            #[allow(clippy::needless_range_loop)]
            for i in 0..HASH_STACK.len() {
                for k in 2..HASH_STACK[0][0].len() {
                    // 0..2 unused
                    for j in 0..HASH_STACK[0].len() {
                        // Iterate over heights first (patterns combine heights)
                        HASH_STACK[i][j][k] = if k < 4 {
                            // Generate hash value for the given square-height-color
                            rng.gen()
                        } else {
                            // Combine hashes
                            let mut stack = Stack::from_raw(k as _);
                            let top = Stack::from_hand_and_count(stack.take(1), 1);

                            let top_j = j + stack.height() as usize;
                            if top_j >= HASH_STACK[i].len() {
                                continue;
                            }

                            HASH_STACK[i][j][stack.raw() as usize]
                                ^ HASH_STACK[i][top_j][top.raw() as usize]
                        };
                    }
                }
            }
        }

        *init = true;
    }
}

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
            Left | Down => !(!(0 as Bitboard) >> 1).wrapping_shr(ray_hits.leading_zeros()),
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

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            (b'a' + self.col_index() as u8) as char,
            self.row_index() + 1,
        )
    }
}

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
    fn shift(self, amount: usize, dir: Direction) -> Self {
        sq(match dir {
            Right => self.0 + amount,
            Up => self.0 + amount * ROW_LEN,
            Left => self.0 - amount,
            Down => self.0 - amount * ROW_LEN,
        })
    }

    #[must_use]
    fn row_index(self) -> usize {
        self.0 / ROW_LEN
    }

    #[must_use]
    fn col_index(self) -> usize {
        self.0 % ROW_LEN
    }

    #[must_use]
    fn row(self) -> Self {
        sq(self.row_index() * ROW_LEN)
    }

    #[must_use]
    fn col(self) -> Self {
        sq(self.col_index())
    }
}

fn sq(sq: usize) -> Square {
    debug_assert!(sq < ARR_LEN);
    debug_assert!(sq % ROW_LEN < SIZE);

    Square(sq)
}

fn bit_squares(bitboard: Bitboard) -> impl Iterator<Item = Square> {
    Bits::new([bitboard]).map(|s| sq(s as usize))
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Pattern(u32);

impl Pattern {
    fn execute(self) -> (u32, DropCounts) {
        let mut dc = DropCounts(self.0 | 1 << HAND);
        // TODO: Investigate unwrap
        (HAND - dc.next().unwrap(), dc)
    }
}

fn pat(pat: u32) -> Pattern {
    debug_assert!(pat > 0);
    debug_assert!(pat < 1 << HAND);

    Pattern(pat)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Action(ActionBacking);

impl crate::game::Action for Action {}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.branch(
            f,
            |f| f.write_str("<pass>"),
            |f, sq, piece| write!(f, "{piece}{sq}"),
            |f, sq, dir, pat| {
                let (taken, counts) = pat.execute();
                if taken == 1 {
                    write!(f, "{sq}{dir}")
                } else if counts.count() == 1 {
                    write!(f, "{taken}{sq}{dir}")
                } else {
                    write!(f, "{taken}{sq}{dir}{counts}")
                }
            },
        )
    }
}

impl Default for Action {
    fn default() -> Self {
        Self::pass()
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
#[derive(Debug)]
pub struct State {
    road: Pair<Bitboard>,
    block: Pair<Bitboard>,

    stones_left: Pair<u32>,
    caps_left: Pair<u32>,

    ply: u32,
    last_reversible: u32,

    stacks: [Stack; ARR_LEN],
    hashes: Pair<WrappingArray<Hash, HIST_LEN>>,

    killers: WrappingArray<Action, 32>,
}

impl Default for State {
    fn default() -> Self {
        Self::new(Options::from_position(Position::Start(SIZE)).unwrap()).unwrap()
    }
}

impl State {
    pub(crate) fn new(opt: Options) -> Result<Self, NewGameError> {
        init();

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
            last_reversible: ply,
            stacks,
            hashes: Pair::both(WrappingArray(Default::default())),
            killers: WrappingArray(Default::default()),
        })
    }

    fn color(&self) -> bool {
        self.ply & 1 != 0
    }

    fn is_opening(&self) -> bool {
        self.ply < 2
    }

    fn hash_mut(&mut self) -> &mut Hash {
        let color = self.color();
        &mut self.hashes[color][self.ply / 2]
    }

    // Performance experiment: swap C and &mut Self.
    // Results: insignificant, try again later.
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

        'skip_nobles: {
            let mut for_placements = {
                #[inline(always)]
                |acc, piece| {
                    bit_squares(empty).try_fold(
                        acc,
                        #[inline(always)]
                        |acc, sq| f(acc, self, Action::place(sq, piece)),
                    )
                }
            };

            if has_stones {
                acc = for_placements(acc, Piece::Flat)?;

                if is_opening {
                    break 'skip_nobles;
                }

                acc = for_placements(acc, Piece::Wall)?;
            }

            if has_caps {
                acc = for_placements(acc, Piece::Cap)?;
            }
        }

        if !is_opening {
            for src in bit_squares(own) {
                let is_cap = src.bit() & cap != 0;

                let max_pieces = self.stacks[src].height().min(HAND);
                let start_bit = 1 << HAND >> max_pieces;

                debug_assert_ne!(max_pieces, 0);

                let mut spread = {
                    #[inline(always)]
                    |mut acc, dir| {
                        let ray = ray(src, dir);
                        let ray_hits = ray & block;
                        let ray_hit = closest_hit(ray_hits, dir);

                        let range = if ray_hit != 0 {
                            distance(src, sq(ray_hit.trailing_zeros() as usize), dir) - 1
                        } else {
                            ray.count_ones()
                        };

                        let mut do_spreads = |mut acc, mut pattern, range, limit| {
                            if range > 0 {
                                while pattern < limit {
                                    acc = f(acc, self, Action::spread(src, dir, pat(pattern)))?;

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
            }
        }

        Continue(acc)
    }

    // Performance experiment: remove undo option (always force undo).
    // Results: mixed.
    fn with<R>(&mut self, undo: bool, action: Action, f: impl FnOnce(&mut Self) -> R) -> R {
        let mut s = self;
        let color = s.color() ^ s.is_opening();

        debug_assert!(s.is_legal(action), "{action} for {s:?}");

        let hash = *s.hash_mut() ^ Hash::SIDE_TO_MOVE;

        s.ply += 1;

        let last_reversible = s.last_reversible;

        let r = action.branch(
            (&mut s, hash, f),
            |(s, _, f)| f(s),
            |(s, mut hash, f), sq, piece| {
                let bit = sq.bit();

                // Placement is not reversible
                s.last_reversible = s.ply;

                if piece.is_road() {
                    s.road[color] ^= bit;
                }

                if piece.is_block() {
                    s.block[color] ^= bit;

                    hash ^= if piece.is_road() {
                        unsafe { HASH_CAP[sq] }
                    } else {
                        unsafe { HASH_WALL[sq] }
                    };
                }

                if piece.is_stone() {
                    s.stones_left[color] -= 1;
                } else {
                    s.caps_left[color] -= 1;
                }

                s.stacks[sq] = Stack::one_tall(color);

                hash ^= unsafe { HASH_STACK[sq][0][s.stacks[sq].raw() as usize] };

                *s.hash_mut() = hash;
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
            |(s, mut hash, f), mut sq, dir, pat| {
                let mut bit = sq.bit();

                let road = s.road;
                let block = s.block;
                let stacks = s.stacks;

                let is_road = road[color] & bit != 0;
                let is_block = block[color] & bit != 0;

                let (taken, counts) = pat.execute();

                let mut hand = s.stacks[sq].take(taken);

                // TODO: Inspect bounds checks
                hash ^= unsafe {
                    HASH_STACK[sq][s.stacks[sq].height() as usize]
                        [Stack::from_hand_and_count(hand, taken).raw() as usize]
                };

                let top = s.stacks[sq].top();

                if top.map(|new_color| new_color != color).unwrap_or(true) {
                    s.road[color] &= !bit;
                }
                if let Some(new_color) = top {
                    s.road[new_color] |= bit;
                }

                if is_block {
                    s.block[color] &= !bit;

                    hash ^= if is_road {
                        unsafe { HASH_CAP[sq] }
                    } else {
                        unsafe { HASH_WALL[sq] }
                    };
                }

                for count in counts {
                    sq = sq.shift(1, dir);
                    bit = sq.bit();

                    // TODO: Inspect bounds checks
                    // FIXME: This is not good
                    // - manually masks off the garbage bits with a magic number
                    // - doesn't reuse data computed within the following .drop()
                    hash ^= unsafe {
                        HASH_STACK[sq][s.stacks[sq].height() as usize]
                            [(Stack::from_hand_and_count(hand, count).raw() & 0x1FF) as usize]
                    };

                    s.stacks[sq].drop(&mut hand, count);

                    s.road.white &= !bit;
                    s.road.black &= !bit;

                    s.road[s.stacks[sq].top_unchecked()] |= bit;
                }

                if is_block {
                    if is_road {
                        if (s.block.white | s.block.black) & bit != 0 {
                            // Smashing a wall is not reversible
                            s.last_reversible = s.ply;

                            hash ^= unsafe { HASH_WALL[sq] };
                        }

                        // Maybe smash opponent wall
                        // No need to unset block if smashing own wall
                        s.block[!color] &= !bit;

                        hash ^= unsafe { HASH_CAP[sq] };
                    } else {
                        // Unset own road bit, which was speculatively set in the loop
                        // Opponent's bit has already been unset in the loop
                        s.road[color] &= !bit;

                        hash ^= unsafe { HASH_WALL[sq] };
                    }
                    s.block[color] |= bit;
                }

                *s.hash_mut() = hash;
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
            s.last_reversible = last_reversible;
            s.ply -= 1;
        }

        r
    }

    // Performance experiment: use a Status enum.
    // Results: mixed, try again later.

    // Performance experiment: swap S and &mut Self.
    // Results: insignificant, try again later.
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
            || self.road.white | self.road.black | self.block.white | self.block.black == BOARD
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
            let next = curr.map(|c| (c | c << 1 | c >> 1 | c << ROW_LEN | c >> ROW_LEN) & road);

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

    /// Assumes that there exists at least one [`State`] for which the [`Action`] is valid.
    fn is_legal(&mut self, action: Action) -> bool {
        let color = self.color();
        let opening = self.is_opening();
        let clever = action.branch(
            (),
            |_| false,
            |_, sq, piece| {
                (!opening || piece.is_flat())
                    && self.stacks[sq].is_empty()
                    && if piece.is_stone() {
                        self.stones_left[color] != 0
                    } else {
                        self.caps_left[color] != 0
                    }
            },
            |_, sq, dir, pat| {
                !opening && {
                    let (taken, counts) = pat.execute();
                    self.stacks[sq].height() >= taken
                        && self.stacks[sq].top_unchecked() == color
                        && {
                            let range = counts.count();
                            let end_sq = sq.shift(range, dir);

                            let span_exclusive = ray(sq, dir) & ray(end_sq, -dir);
                            let span = span_exclusive | end_sq.bit();

                            let block = self.block.white | self.block.black;

                            // TODO: Investigate unwrap
                            span & block == 0
                                || span_exclusive & block == 0 && counts.last().unwrap() == 1 && {
                                    let road = self.road.white | self.road.black;
                                    let cap = road & block;

                                    cap & end_sq.bit() == 0 && cap & sq.bit() != 0
                                }
                        }
                }
            },
        );

        debug_assert_eq!(
            self.for_actions((), |_, _, other| {
                if action == other {
                    Break(())
                } else {
                    Continue(())
                }
            })
            .is_break(),
            clever,
            "{action} is legal({clever}) for {self:?}"
        );
        clever
    }

    fn eval(&self) -> Eval {
        let eval_half =
            |color| self.stones_left[color] as i32 * -20 + self.count_flats(color) as i32 * 14;

        let color = self.color();
        Eval::new(eval_half(color) - eval_half(!color) + 17)
    }

    fn search(&mut self, depth: u32, mut alpha: Eval, beta: Eval) -> (Eval, Option<Action>) {
        self.status(
            (),
            |_, s| {
                if depth == 0 {
                    return (s.eval(), None);
                }

                let mut best_score = -Eval::MAX;
                let mut best_action = None;

                let killer = s.killers[s.ply];

                let mut f = {
                    #[inline(always)]
                    |s: &mut Self, action| {
                        let score = -s
                            .with(true, action, |s| s.search(depth - 1, -beta, -alpha))
                            .0;

                        if score > best_score {
                            best_score = score;
                            best_action = Some(action);
                            if score > alpha {
                                alpha = score;
                                if alpha >= beta {
                                    s.killers[s.ply] = action;
                                    return Break(());
                                }
                            }
                        }

                        Continue(())
                    }
                };

                'ret: {
                    if s.is_legal(killer) && f(s, killer).is_break() {
                        break 'ret;
                    }

                    s.for_actions((), |_, s, action| {
                        if action == killer {
                            Continue(())
                        } else {
                            f(s, action)
                        }
                    });
                }

                (best_score, best_action)
            },
            |_, _| (Eval::ZERO, None),
            |_, s| (Eval::win(s.ply), None),
            |_, s| (Eval::loss(s.ply), None),
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
                        Continue(sum + s.with(true, action, |s| s.perft(depth - 1, mode)))
                    })
                    .into_continue()
                },
                |_, _| 1,
                |_, _| 1,
                |_, _| 1,
            ),
        }
    }

    fn search(&mut self, depth: u32) -> (Eval, Option<Box<dyn crate::game::Action>>) {
        let (score, action) = self.search(depth, -Eval::DECISIVE, Eval::DECISIVE);
        (score, action.map(|action| Box::new(action) as _))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft() {
        for &(depth, expected) in PERFT {
            println!("running perft {depth}...");
            assert_eq!(State::default().perft(depth, PerftMode::Batch), expected);
        }
    }
}

// $end_template

use crate::size::size6::*;
