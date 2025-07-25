use crate::*;

#[repr(C)]
#[derive(Debug)]
pub struct State {
    pub(crate) road: Pair<Bitboard>,
    pub(crate) block: Pair<Bitboard>,

    pub(crate) stones_left: Pair<u32>,
    pub(crate) caps_left: Pair<u32>,

    pub(crate) nodes: u64,
    pub(crate) generation: u32,

    pub(crate) half_komi: i32,
    pub(crate) ply: u32,
    pub(crate) last_reversible: u32,

    pub(crate) abort: Arc<AtomicBool>,
    pub(crate) abort_inactive: Arc<AtomicBool>,

    pub(crate) stacks: [Stack; ARR_LEN],

    pub(crate) influence: Pair<Influence>,

    pub(crate) hashes: WrappingArray<Hash, MAX_DEPTH>,
    pub(crate) killers: WrappingArray<Action, MAX_DEPTH>,

    pub(crate) tt: Box<[TtBucket]>,

    pub(crate) search: SearchParamsProvider,
    pub(crate) eval: EvalParamsProvider,
}

impl State {
    pub fn new(opt: Options) -> Result<Self, NewGameError> {
        if !opt.params.tt_size.is_power_of_two() {
            return Err(NewGameError);
        }

        init();

        Ok(Self {
            road: Pair::default(),
            block: Pair::default(),
            stones_left: opt.start_stones,
            caps_left: opt.start_caps,
            nodes: 0,
            generation: 0,
            half_komi: opt.half_komi,
            ply: 0,
            last_reversible: 0,
            abort: Arc::new(AtomicBool::new(false)),
            abort_inactive: Arc::new(AtomicBool::new(false)),
            stacks: [Stack::EMPTY; ARR_LEN],
            influence: Pair::both(Influence::EDGES),
            hashes: WrappingArray([Hash::ZERO; MAX_DEPTH]),
            killers: WrappingArray([Action::PASS; MAX_DEPTH]),
            tt: std::iter::repeat_n(TtBucket::default(), opt.params.tt_size)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            search: SearchParamsProvider::new(opt.params).ok_or(NewGameError)?,
            eval: EvalParamsProvider::new(EVAL_PARAMS).ok_or(NewGameError)?,
        })
    }

    pub(crate) fn search(
        &mut self,
        depth: u32,
        mut alpha: Eval,
        mut beta: Eval,
        allow_nmp: bool,
    ) -> Eval {
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
                let mut best_action = Action::PASS;

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
                            Action::PASS
                        };

                        let nmp_factor = s.search.nmp_factor;
                        if depth > nmp_factor
                            && allow_nmp
                            && nmp_factor != 0
                            && s.eval() + s.search.nmp_fudge + s.search.nmp_eval_margin >= beta
                        {
                            // NMP conditions
                            // - depth doesn't underflow
                            // - parent allows (currently only in scout search)
                            // - NMP enabled
                            // - eval is high
                            let score = -s.with(true, Action::PASS, |s| {
                                s.search(depth - nmp_factor - 1, -beta, -beta + 1, false)
                            });

                            if score + s.search.nmp_fudge >= beta {
                                return beta;
                            }
                        }

                        let mut allow_scout_window = false;
                        let mut f = |s: &mut Self, action| {
                            if s.abort.load(Relaxed) {
                                return Break(());
                            }

                            let mut score;
                            'skip_full_window: {
                                if allow_scout_window {
                                    score = -s.with(true, action, |s| {
                                        s.search(depth - 1, -alpha - 1, -alpha, true)
                                    });

                                    if score <= alpha || score >= beta {
                                        break 'skip_full_window;
                                    }
                                }

                                allow_scout_window = s.search.use_pvs;
                                score = -s.with(true, action, |s| {
                                    s.search(depth - 1, -beta, -alpha, false)
                                });
                            }

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

                        _ = s.for_actions((), |_, s, action| {
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
                        >= rate_entry(entry.depth, entry.packed.generation(), s.generation)
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

    pub(crate) fn eval(&self) -> Eval {
        let eval_half = |color| {
            let inf = self.influence[color];
            let (my_road, opp_road) = self.road.get(color);
            let (my_block, opp_block) = self.block.get(color);

            let my_wall = !my_road & my_block;
            let opp_piece = opp_road | opp_block;
            // let opp_flat = opp_road & !opp_block;

            let opp_supports = opp_piece | my_wall;

            let right = opp_supports.shift(Right);
            let up = opp_supports.shift(Up);
            let left = opp_supports.shift(Left);
            let down = opp_supports.shift(Down);

            let edge_right = right | EDGE_LEFT;
            let edge_up = up | EDGE_BOTTOM;
            let edge_left = left | EDGE_RIGHT;
            let edge_down = down | EDGE_TOP;

            let prot_hard_horz = edge_right & edge_left;
            let prot_hard_vert = edge_up & edge_down;

            let prot_soft_horz = edge_right | edge_left;
            let prot_soft_vert = edge_up | edge_down;

            let prot_soft_no_edge_horz = right | left;
            let prot_soft_no_edge_vert = up | down;

            let blocked_horz = prot_hard_vert | prot_soft_no_edge_horz & prot_soft_vert;
            let blocked_vert = prot_hard_horz | prot_soft_no_edge_vert & prot_soft_horz;

            let nontraversable = my_wall | opp_block;

            let traversable_horz = BOARD ^ (nontraversable | opp_road & blocked_horz);
            let traversable_vert = BOARD ^ (nontraversable | opp_road & blocked_vert);

            let dist_horz = self.flood_distance(inf[LEFT], inf[RIGHT], traversable_horz, my_road);
            let dist_vert = self.flood_distance(inf[BOTTOM], inf[TOP], traversable_vert, my_road);

            let total_dist = dist_horz + dist_vert;
            let smaller_dist = min(dist_horz, dist_vert);

            self.stones_left[color] as i32 * self.eval.stones_left
                + self.caps_left[color] as i32 * self.eval.caps_left
                + total_dist * self.eval.total_dist
                + smaller_dist * self.eval.smallest_dist
        };

        let color = self.active_color();
        let sides = eval_half(color) - eval_half(!color);
        Eval::new(
            sides * 2
                + self.half_flat_count_diff() * sign(color) * self.eval.flat_count
                + self.eval.side_to_move,
        )
    }

    // Performance experiment: swap C and &mut Self.
    // Results: insignificant, try again later.
    pub(crate) fn for_actions<B, C>(
        &mut self,
        mut acc: C,
        mut f: impl FnMut(C, &mut Self, Action) -> ControlFlow<B, C>,
    ) -> ControlFlow<B, C> {
        let color = self.active_color();

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
    pub(crate) fn with<R>(
        &mut self,
        undo: bool,
        action: Action,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let mut s = self;
        let color = s.active_color() ^ s.is_opening();

        debug_assert!(
            action == Action::PASS || s.is_legal(action),
            "{action} for {s:?}",
        );

        let hash = *s.hash_mut() ^ Hash::SIDE_TO_MOVE;

        s.ply += 1;

        let last_reversible = s.last_reversible;

        let r = action.branch(
            (&mut s, hash, f),
            |(s, hash, f)| {
                *s.hash_mut() = hash;
                f(s)
            },
            |(s, mut hash, f), sq, piece| {
                let bit = sq.bit();

                let influence = s.influence[color];

                // Placement is not reversible
                s.last_reversible = s.ply;

                if piece.is_road() {
                    s.road[color] ^= bit;
                    s.influence[color].flood(s.road[color], false);
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
                    s.influence[color] = influence;

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
                let influence = s.influence;

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

                for color in [WHITE, BLACK] {
                    if road[color] != s.road[color] {
                        if road[color] & !s.road[color] != 0 {
                            s.influence[color].clear();
                        }

                        s.influence[color].flood(s.road[color], false);
                    }
                }

                *s.hash_mut() = hash;
                let r = f(s);

                if undo {
                    s.influence = influence;
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

    /// Assumes that there exists at least one [`State`] for which the [`Action`] is valid.
    pub(crate) fn is_legal(&mut self, action: Action) -> bool {
        let color = self.active_color();
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

    // Performance experiment: use a Status enum.
    // Results: mixed, try again later.

    // Performance experiment: swap S and &mut Self.
    // Results: insignificant, try again later.
    pub(crate) fn status<S, R>(
        &mut self,
        state: S,
        ongoing: impl FnOnce(S, &mut Self) -> R,
        draw: impl FnOnce(S, &mut Self) -> R,
        win: impl FnOnce(S, &mut Self) -> R,
        loss: impl FnOnce(S, &mut Self) -> R,
    ) -> R {
        let color = self.active_color();

        if self.has_road(!color) {
            return loss(state, self);
        }

        if self.has_road(color) {
            return win(state, self);
        }

        if self.stones_left[!color] == 0 && self.caps_left[!color] == 0
            || self.road.white | self.road.black | self.block.white | self.block.black == BOARD
        {
            let diff = self.half_flat_count_diff() * sign(color);

            return if diff < 0 {
                loss(state, self)
            } else if diff > 0 {
                win(state, self)
            } else {
                draw(state, self)
            };
        }

        ongoing(state, self)
    }

    pub(crate) fn flood_distance(
        &self,
        start: Bitboard,
        goal: Bitboard,
        traversable: Bitboard,
        fast: Bitboard,
    ) -> i32 {
        let max_dist = SIZE as i32 + self.eval.max_dist_offset;

        let mut c = start & traversable;
        if c & goal != 0 {
            return 0;
        }

        for cost in 1..max_dist {
            // Spread to traversable neighbors
            let mut nc = c.spread() & traversable | c;

            if nc & goal != 0 {
                return cost;
            }

            if c == nc {
                // If no more traversable neighbors, no road possible
                break;
            }

            loop {
                let new_fast = nc & !c & fast;
                c = nc;

                if new_fast == 0 {
                    break;
                }

                nc |= new_fast.spread() & traversable;

                if nc & goal != 0 {
                    return cost;
                }
            }
        }

        max_dist
    }

    #[inline]
    pub(crate) fn has_road(&self, color: bool) -> bool {
        self.influence[color].intersections_of_opposites() & self.road[color] != 0
    }

    #[inline]
    pub(crate) fn half_flat_count_diff(&self) -> i32 {
        (self.road.white & !self.block.white).count_ones() as i32 * 2
            - (self.road.black & !self.block.black).count_ones() as i32 * 2
            - self.half_komi
    }

    #[inline]
    pub(crate) fn hash_mut(&mut self) -> &mut Hash {
        &mut self.hashes[self.ply]
    }
}

impl Game for State {
    fn search(&mut self, depth: u32) -> Option<(Eval, Box<dyn Move>)> {
        assert!(depth > 0);
        assert!(depth < MAX_DEPTH as _);

        let (idx, sig) = self.hash_mut().split(self.tt.len());

        'skip_full_window: {
            if let Some(&mut TtEntry {
                score: expected_score,
                packed,
                ..
            }) = self.tt[idx].entry(sig)
            {
                if packed.is_exact() {
                    let mut alpha_margin = self.search.aspiration_window;
                    let mut beta_margin = self.search.aspiration_window;

                    for _ in 0..self.search.aspiration_attempts {
                        let alpha = expected_score - alpha_margin;
                        let beta = expected_score + beta_margin;

                        let score = self.search(depth, alpha, beta, false);

                        if score > alpha && score < beta {
                            break 'skip_full_window;
                        }

                        if score <= alpha {
                            alpha_margin *= self.search.aspiration_scaling;
                        }
                        if score >= beta {
                            beta_margin *= self.search.aspiration_scaling;
                        }
                    }
                }
            }

            self.search(depth, -Eval::DECISIVE, Eval::DECISIVE, false);
        }

        self.generation += 1;

        self.tt[idx]
            .entry(sig)
            .map(|entry| (entry.score, Box::new(entry.action) as Box<dyn Move>))
    }

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

    fn parser(&mut self) -> fn(&str) -> Result<Box<dyn Move>, ParseMoveError> {
        |mv| {
            // TODO: Remove
            // NOTE: Requires std
            use takparse::{Direction, Move, MoveKind::*, Piece};

            let Ok(mv) = mv.parse::<Move>() else {
                return Err(ParseMoveError);
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
    }

    fn play(&mut self, action: Box<dyn Move>) -> Result<(), PlayMoveError> {
        let action = action.as_any();
        let Some(&action) = action.downcast_ref() else {
            panic!("action-state size mismatch");
        };

        if self.is_legal(action) {
            self.with(false, action, |_| ());
            Ok(())
        } else {
            Err(PlayMoveError)
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

        self.influence.white.clear_and_flood(self.road.white, false);
        self.influence.black.clear_and_flood(self.road.black, false);

        self.ply = tps.ply() as u32;

        Ok(())
    }

    fn pv(&mut self) -> Box<dyn fmt::Display + '_> {
        Box::new(Pv::new(self))
    }

    fn abort_flag(&self) -> AbortFlag {
        AbortFlag::new(&self.abort)
    }

    fn clear_abort_flag(&self) -> bool {
        self.abort
            .compare_exchange(true, false, Relaxed, Relaxed)
            .is_ok()
    }

    fn swap_abort_flags(&mut self) {
        core::mem::swap(&mut self.abort, &mut self.abort_inactive);
    }

    fn nodes(&self) -> u64 {
        self.nodes
    }

    fn clear_nodes(&mut self) {
        self.nodes = 0;
    }

    fn hash(&mut self) -> Hash {
        *self.hash_mut()
    }

    fn stones_left(&self) -> Pair<u32> {
        self.stones_left
    }

    fn caps_left(&self) -> Pair<u32> {
        self.caps_left
    }

    fn active_color(&self) -> bool {
        self.ply & 1 != 0
    }

    fn is_opening(&self) -> bool {
        self.ply < 2
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new(Options::default(SIZE).unwrap()).unwrap()
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
