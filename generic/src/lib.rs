#![allow(
	clippy::precedence, // Personal opinion
	clippy::comparison_chain, // Required for optimal performance at the time of writing
)]

extern crate alloc;

mod lut;
mod prelude;

use crate::prelude::*;

use alloc::sync::Arc;
use core::{
    any::Any,
    fmt,
    mem::transmute,
    ops::{
        ControlFlow::{self, *},
        Index, IndexMut,
    },
    sync::atomic::{AtomicBool, Ordering::Relaxed},
};

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

impl GameAction for Action {
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

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

    // `self.0 as u32` is unnecessary iff Action is backed by u32
    #[allow(clippy::unnecessary_cast)]
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

#[derive(Debug, Clone, Copy, Default)]
struct Packed(u8);

impl Packed {
    fn is_upper(self) -> bool {
        self.0 & 0x40 == 0
    }

    fn is_lower(self) -> bool {
        self.0 & 0x80 == 0
    }

    fn generation(self) -> u32 {
        self.0 as u32 & 0x3F
    }

    fn set_upper(&mut self) {
        self.0 |= 0x80;
    }

    fn set_lower(&mut self) {
        self.0 |= 0x40;
    }

    fn set_generation(&mut self, generation: u32) {
        self.0 = self.0 & !0x3F | generation as u8 & 0x3F;
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct TtEntry {
    sig: u64,
    score: Eval,
    action: Action,
    depth: u8,
    packed: Packed,
}

// TODO: Cleanup
fn rate_entry(depth: u8, entry_gen: u32, curr_gen: u32) -> i32 {
    depth as i32 - (curr_gen - entry_gen & 0x3F) as i32
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(align(32))]
struct TtBucket([TtEntry; 2]);

impl TtBucket {
    fn entry(&mut self, sig: u64) -> Option<&mut TtEntry> {
        self.0.iter_mut().find(|e| e.sig == sig)
    }

    fn worst_entry(&mut self, curr_gen: u32) -> &mut TtEntry {
        self.0
            .iter_mut()
            .min_by_key(|e| rate_entry(e.depth, e.packed.generation(), curr_gen))
            .unwrap()
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

    nodes: u64,
    generation: u32,

    abort: Arc<AtomicBool>,
    abort_inactive: Arc<AtomicBool>,

    stacks: [Stack; ARR_LEN],
    hashes: Pair<WrappingArray<Hash, HIST_LEN>>,

    killers: WrappingArray<Action, 32>,

    tt: Box<[TtBucket]>,
}

impl Default for State {
    fn default() -> Self {
        Self::new(Options::default(SIZE).unwrap()).unwrap()
    }
}

impl State {
    pub fn new(opt: Options) -> Result<Self, NewGameError> {
        if opt.half_komi != 0 {
            return Err(NewGameError);
        }

        if !opt.tt_size.is_power_of_two() {
            return Err(NewGameError);
        }

        init();

        Ok(Self {
            road: Pair::default(),
            block: Pair::default(),
            stones_left: opt.start_stones,
            caps_left: opt.start_caps,
            ply: 0,
            last_reversible: 0,
            nodes: 0,
            generation: 0,
            abort: Arc::new(AtomicBool::new(false)),
            abort_inactive: Arc::new(AtomicBool::new(false)),
            stacks: [Stack::EMPTY; ARR_LEN],
            hashes: Pair::both(WrappingArray(Default::default())),
            killers: WrappingArray(Default::default()),
            tt: core::iter::repeat(TtBucket::default())
                .take(opt.tt_size)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
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
            let mut for_placements = |acc, piece| {
                bit_squares(empty).try_fold(acc, |acc, sq| f(acc, self, Action::place(sq, piece)))
            };

            if has_stones {
                acc = for_placements(acc, Flat)?;

                if is_opening {
                    break 'skip_nobles;
                }

                acc = for_placements(acc, Wall)?;
            }

            if has_caps {
                acc = for_placements(acc, Cap)?;
            }
        }

        if !is_opening {
            for src in bit_squares(own) {
                let is_cap = src.bit() & cap != 0;

                let max_pieces = self.stacks[src].height().min(HAND);
                let start_bit = 1 << HAND >> max_pieces;

                debug_assert_ne!(max_pieces, 0);

                let mut spread = |mut acc, dir| {
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
                    // - manually masks off the garbage bits with a tangentially related number
                    // - doesn't reuse data computed within the following .drop()
                    hash ^= unsafe {
                        HASH_STACK[sq][s.stacks[sq].height() as usize]
                            [(Stack::from_hand_and_count(hand, count).raw() % (2 << HAND)) as usize]
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
        let eval_half = |color| {
            self.stones_left[color] as i32 * -20
                + self.caps_left[color] as i32 * -30
                + self.count_flats(color) as i32 * 14
        };

        let color = self.color();
        Eval::new(eval_half(color) - eval_half(!color) + 17)
    }

    fn search(&mut self, depth: u32, mut alpha: Eval, mut beta: Eval) -> Eval {
        self.nodes += 1;
        self.status(
            (),
            |_, s| {
                if depth == 0 {
                    return s.eval();
                }

                let original_alpha = alpha;
                let original_beta = beta;

                let mut best_score = -Eval::MAX;
                let mut best_action = Action::pass();

                let (idx, sig) = s.hash_mut().split(s.tt.len());
                'ret: {
                    'update_tt: {
                        let bucket = &mut s.tt[idx];
                        let entry = bucket.entry(sig);

                        let tt_action = if let Some(entry) = entry {
                            if entry.depth as u32 == depth {
                                let score = entry.score;

                                if entry.packed.is_lower() {
                                    alpha = alpha.max(score);
                                }
                                if entry.packed.is_upper() {
                                    beta = beta.min(score);
                                }

                                if alpha >= beta {
                                    best_score = score;
                                    entry.packed.set_generation(s.generation);
                                    break 'ret;
                                }
                            }

                            entry.action
                        } else {
                            Action::pass()
                        };

                        let mut f = |s: &mut Self, action| {
                            if s.abort.load(Relaxed) {
                                return Break(());
                            }

                            let score =
                                -s.with(true, action, |s| s.search(depth - 1, -beta, -alpha));

                            if score > best_score {
                                best_score = score;
                                best_action = action;
                                if score > alpha {
                                    alpha = score;
                                    if alpha >= beta {
                                        s.killers[s.ply] = action;
                                        return Break(());
                                    }
                                }
                            }

                            Continue(())
                        };

                        if s.is_legal(tt_action) && f(s, tt_action).is_break() {
                            break 'update_tt;
                        }

                        let killer = s.killers[s.ply];

                        if tt_action != killer && s.is_legal(killer) && f(s, killer).is_break() {
                            break 'update_tt;
                        }

                        s.for_actions((), |_, s, action| {
                            if action == tt_action || action == killer {
                                Continue(())
                            } else {
                                f(s, action)
                            }
                        });
                    }

                    if s.abort.load(Relaxed) {
                        break 'ret;
                    }

                    let bucket = &mut s.tt[idx];
                    let entry = if let Some(entry) = bucket.entry(sig) {
                        entry
                    } else {
                        bucket.worst_entry(s.generation)
                    };

                    if rate_entry(depth as _, s.generation, s.generation)
                        > rate_entry(entry.depth, entry.packed.generation(), s.generation)
                    {
                        entry.sig = sig;
                        entry.score = best_score;
                        entry.action = best_action;
                        entry.depth = depth as _;

                        entry.packed = Packed::default();
                        entry.packed.set_generation(s.generation);
                        if best_score <= original_alpha {
                            entry.packed.set_upper();
                        }
                        if best_score >= original_beta {
                            entry.packed.set_lower();
                        }
                    }
                }

                best_score
            },
            |_, _| Eval::ZERO,
            |_, s| Eval::win(s.ply),
            |_, s| Eval::loss(s.ply),
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

    fn search(&mut self, depth: u32) -> (Eval, Box<dyn GameAction>) {
        assert!(depth > 0);

        self.search(depth, -Eval::DECISIVE, Eval::DECISIVE);

        self.generation += 1;

        let (idx, sig) = self.hash_mut().split(self.tt.len());
        let bucket = &mut self.tt[idx];
        let entry = bucket.entry(sig).unwrap();

        (entry.score, Box::new(entry.action))
    }

    fn parse_action(&mut self, ptn: &str) -> Result<Box<dyn GameAction>, ParseActionError> {
        // TODO: Remove
        // NOTE: Requires std
        use takparse::{Direction, Move, MoveKind::*, Piece};

        let Ok(mv) = ptn.parse::<Move>() else {
            return Err(ParseActionError);
        };

        let square = mv.square();
        let sq = sq(square.column() as usize + square.row() as usize * ROW_LEN);

        Ok(Box::new(match mv.kind() {
            Place(piece) => Action::place(
                sq,
                match piece {
                    Piece::Flat => Flat,
                    Piece::Wall => Wall,
                    Piece::Cap => Cap,
                },
            ),
            Spread(direction, pattern) => Action::spread(
                sq,
                match direction {
                    Direction::Up => Up,
                    Direction::Down => Down,
                    Direction::Right => Right,
                    Direction::Left => Left,
                },
                pat(pattern.mask() as u32 >> 8 - HAND),
            ),
        }))
    }

    fn play(&mut self, action: Box<dyn GameAction>) -> Result<(), PlayActionError> {
        let action = action.as_any();
        let Some(&action) = action.downcast_ref() else {
            panic!("action-state size mismatch");
        };

        if self.is_legal(action) {
            self.with(false, action, |_| ());
            Ok(())
        } else {
            Err(PlayActionError)
        }
    }

    fn set_position(&mut self, tps: &str) -> Result<(), SetPositionError> {
        // TODO: Remove
        // NOTE: Requires std
        use takparse::{Color, Piece, Tps};

        if self.ply != 0 {
            todo!()
        }

        let tps: Tps = tps.parse().map_err(|_| SetPositionError)?;
        for (row, y) in tps.board_2d().zip((0..SIZE).rev()) {
            for (stack, x) in row.zip(0..SIZE) {
                if let Some(stack) = stack {
                    let sq = sq(x + y * ROW_LEN);

                    for color in stack.colors() {
                        let color = color != Color::White;
                        self.stacks[sq].drop(&mut Hand::one_piece(color), 1);
                        self.stones_left[color] -= 1;
                    }

                    let top = stack.top();
                    let color = stack.top_color() != Color::White;

                    if matches!(top, Piece::Flat | Piece::Cap) {
                        self.road[color] |= sq.bit();
                    }
                    if matches!(top, Piece::Wall | Piece::Cap) {
                        self.block[color] |= sq.bit();
                    }

                    if top == Piece::Cap {
                        self.stones_left[color] += 1; // Correct overcounting from the stack
                        self.caps_left[color] -= 1;
                    }
                }
            }
        }

        self.ply = tps.ply() as u32;

        Ok(())
    }

    fn take_nodes(&mut self) -> u64 {
        (self.nodes, self.nodes = 0).0
    }

    fn curr_hash(&mut self) -> Hash {
        *self.hash_mut()
    }

    fn abort_flag(&mut self) -> AbortFlag {
        AbortFlag::new(&self.abort)
    }

    fn clear_abort_flag(&mut self) -> bool {
        self.abort
            .compare_exchange(true, false, Relaxed, Relaxed)
            .is_ok()
    }

    fn swap_abort_flags(&mut self) {
        core::mem::swap(&mut self.abort, &mut self.abort_inactive);
    }

    fn stones_left(&mut self) -> Pair<u32> {
        self.stones_left
    }

    fn caps_left(&mut self) -> Pair<u32> {
        self.caps_left
    }

    fn active_color(&mut self) -> bool {
        self.color()
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
