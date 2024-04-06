use crate::*;

impl<'a> State<'a> {
    #[inline]
    pub(crate) fn for_actions<B, C>(
        &mut self,
        mut acc: C,
        f: fn(State, C, u16) -> ControlFlow<B, C>,
    ) -> ControlFlow<B, C> {
        let empty = self.piece() ^ BOARD;
        macro_rules! for_placements {
            ($pc:expr, $e:expr) => {
                let mut left = empty;
                // Assume actions are not generated on completed games.
                loop {
                    log!(self, $e);
                    acc = f(self.lend(), acc, left.trailing_zeros() as u16 | $pc << TAG_OFFSET)?;
                    left &= left - 1;
                    if left == 0 {
                        break;
                    }
                }
            };
        }

        // Only do stone placements if we have any stones in reserve. We're technically checking
        // the wrong reserves on the first move because we assume this passes anyway.
        if self.update.stones_left[self.player()] > 0 {
            for_placements!(FLAT_TAG, GenPlaceFlat);

            // If it's the first move, skip over all the other kinds of actions. This check gets
            // skipped if we don't have stones, but we again don't care.
            if self.copy.ply < 2 {
                return Continue(acc);
            }

            for_placements!(WALL_TAG, GenPlaceWall);
        }

        if self.update.caps_left[self.player()] > 0 {
            for_placements!(CAP_TAG, GenPlaceCap);
        }

        Continue(acc)
    }

    #[no_mangle]
    #[inline]
    pub(crate) fn make(&mut self, new: &mut Out<CopyState>, unmake: &mut Out<Unmake>, action: u16) {
        let new_ply = self.copy.ply + 1;

        let mut hash = self.update.hashes[self.copy.ply] ^ HASH_SIDE_TO_MOVE;

        macro_rules! touch_bucket {
            () => {
                unsafe {
                    // Touch the bucket's cache line.
                    self.update
                        .tt
                        .add(hash as usize & self.update.tt_idx_mask)
                        .cast::<u32>()
                        .read_volatile();
                }
            };
        }

        // Represents the opponent's pieces. No need to add our new pieces, if any.
        let mut opp_own = self.copy.own ^ self.piece();
        let mut new_road = self.copy.road;
        let mut new_noble = self.copy.noble;
        let mut new_tall = self.copy.tall;
        let new_irreversible;
        let new_stack;

        let PovPair {
            my: mut inf,
            opp: mut opp_inf,
        } = *self.copy.influence.pair_mut();

        let mut player = self.player();

        let pat = pat(action);
        let sq = sq(action) as usize;
        if pat == 0 {
            player ^= self.is_first_move();

            new_stack = player as Stack + 2;

            // Update the hash.
            hash ^= hash_stack(sq, new_stack as _, STACK_CAP as _);
            hash ^= hash_sq_pc(action as _);
            touch_bucket!();

            // Remove the appropriate piece from the reserves.
            let pieces_left_ref = &mut (if action >= CAP_TAG << TAG_OFFSET {
                &mut self.update.caps_left
            } else {
                &mut self.update.stones_left
            })[player];
            *pieces_left_ref -= 1;
            unmake.val.kind.place.pieces_left_ptr = (pieces_left_ref as *mut u8).addr();

            let road_bit = (action as Bb >> ROAD_TAG_OFFSET & 1) << sq;
            let simd_road_bit = Simd::splat(road_bit);
            let adjacent = inf & simd_road_bit;

            // Set the road and noble bits, if appropriate piece.
            new_road ^= road_bit;
            new_noble ^= (action as Bb >> NOBLE_TAG_OFFSET & 1) << sq;

            // Placement is irreversible.
            new_irreversible = new_ply;

            // Check if the new road bit is adjacent to any of the influence masks.
            if adjacent.reduce_or() != 0 {
                // Prepare to add the neighbors to the adjacent masks.
                let neighbors = adjacent
                    .simd_ne(Simd::splat(0))
                    .select(simd_road_bit.neighbors(), Simd::splat(0));

                // Add the neighbors.
                let mut old_inf = inf;
                inf |= neighbors;

                // Get a bitboard representing our road pieces.
                let my_road = Simd::splat(new_road & self.copy.own);

                // Expandable neighbors are those that are our road pieces.
                let mut expandable = neighbors & my_road;

                // Count iterations for logging purposes.
                let mut i = 0;

                // If we have any such neighbors that are not part of the old influence, expand.
                // Assume winning placements are rare and/or handled before make.
                while (expandable & !old_inf).reduce_or() != 0 {
                    i += 1;
                    old_inf = inf;
                    inf |= expandable.neighbors();
                    expandable = inf & my_road;
                }

                log!(self, PlacementExpansionIterations(i));
            }

            log!(self, match action >> TAG_OFFSET {
                FLAT_TAG => MakePlaceFlat,
                WALL_TAG => MakePlaceWall,
                CAP_TAG => MakePlaceCap,
                _ => unreachable!(),
            });

            // TODO: Help the compiler by pinning registers with inline assembly.
            // Handle the swap rule.
            if new_ply == 2 {
                (inf, opp_inf) = (opp_inf, inf); // This partially leaks into the normal code path.

                // At this point, opp_own represents the black piece placed by white, but it should
                // represent the white piece placed by black just now. Fix accordingly.
                opp_own ^= opt_fence!(new_road); // Stop LLVM from hoisting this.
            }
        } else {
            // Decode the pattern.
            let mut pat = pat as u32; // u16 operations are slow, and LLVM does not promote them.
            let mut zeros = pat.trailing_zeros();
            let taken = HAND - zeros;

            let stack = *unsafe { self.update.stacks.get_unchecked(sq) };
            new_stack = stack >> taken;
            unmake.val.kind.spread.orig_stack = stack;

            let not_empty = (new_stack != 1) as Bb;

            // Toggle ownership if the top color of the stack changed and the new stack isn't empty.
            // By default, ownership is cleared due to the perspective switch.
            let toggle_ownership = (stack ^ new_stack) as Bb & not_empty;
            let taken_bit = unsafe { unchecked_shl(1, taken) };
            let is_road = new_road >> sq & 1;
            let is_noble = new_noble >> sq & 1;
            let piece_tag = (is_road + 2 * is_noble << TAG_OFFSET) as usize;

            hash ^= hash_stack(
                sq,
                (stack as u32 & taken_bit - 1) + taken_bit,
                new_stack.leading_zeros(), // Base of the removed stack.
            );
            hash ^= hash_sq_pc(sq + piece_tag);
            opp_own ^= toggle_ownership << sq;
            new_road &= !(1 << sq);
            new_road |= not_empty << sq;
            new_noble &= !(1 << sq);
            new_tall &= !(((new_stack < 1 << 2) as Bb) << sq);

            let mut hand = (stack as u32).wrapping_shl(taken.wrapping_neg()); // Avoid u64/u128.

            const STEP_OFFSET_TABLE: u32 =
                1 << 0 | (ROW_LEN as u32) << 8 | 256 - 1 << 16 | (256 - ROW_LEN as u32) << 24;
            let dir = action >> TAG_OFFSET & 3;
            let offset = (STEP_OFFSET_TABLE >> dir * 8) as i8 as isize;
            let mut curr_sq = sq;
            loop {
                curr_sq = curr_sq.wrapping_add_signed(offset);
                pat &= pat - 1;
                if pat == 0 {
                    break;
                }

                // Operate on a transitory stack.
                // The top piece was a flat or nothing, and now becomes a flat.

                let next = pat.trailing_zeros();
                let count = next - zeros;
                zeros = next;

                let stack = unsafe { self.update.stacks.get_unchecked_mut(curr_sq) };
                let rem_cap = stack.leading_zeros();
                let dropped_pieces = hand.wrapping_shr(count.wrapping_neg());
                let dropped_stack = dropped_pieces + 1 << count;
                let shifted_stack = *stack << count;
                let toggle_ownership = (dropped_pieces
                    ^ if *stack == 1 {
                        // If the stack is empty, the own bit is off. If the colors of the topmost
                        // dropped piece and the player don't match, the opponent owns the stack.
                        player as u32
                    } else {
                        // Otherwise, ownership of the stack is transferred iff the colors of the
                        // topmost dropped piece and the previous top of the stack don't match.
                        *stack as u32
                    })
                    & 1;

                *stack = shifted_stack | dropped_pieces as Stack;
                hand <<= count;
                hash ^= hash_stack(curr_sq, dropped_stack, rem_cap);
                opp_own ^= (toggle_ownership as Bb) << curr_sq;
                new_road |= 1 << curr_sq;
                new_tall |= ((*stack >= 1 << 2) as Bb) << curr_sq;
            }

            // Spreads are usually reversible.
            new_irreversible = self.copy.last_irreversible;
        }

        self.update.hashes[new_ply] = hash;
        *unsafe { self.update.stacks.get_unchecked_mut(sq) } = new_stack;
        new.val = CopyState {
            influence: Influence {
                pair: PovPair::new(opp_inf, inf),
            },
            own: opp_own,
            road: new_road,
            noble: new_noble,
            tall: new_tall,
            ply: new_ply,
            last_irreversible: new_irreversible,
        };
    }

    #[no_mangle]
    #[inline]
    pub(crate) fn unmake(&mut self, unmake: &Unmake, action: u16) {
        let mut player = self.player();
        let pat = pat(action);
        let sq = sq(action);
        if pat == 0 {
            player ^= self.is_first_move();
            unsafe {
                *(self.update as *mut UpdateState as *mut u8)
                    .with_addr(unmake.kind.place.pieces_left_ptr) += 1;
                *self.update.stacks.get_unchecked_mut(sq as usize) = 1;
            }
        }
    }
}
