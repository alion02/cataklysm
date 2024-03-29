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
        new.val.ply = new_ply;

        let mut hash = self.update.hashes[self.copy.ply] ^ HASH_SIDE_TO_MOVE;

        let mut player = self.player();
        let pat = pat(action);
        let sq = sq(action);
        if pat == 0 {
            player ^= self.is_first_move();

            let new_stack = player as Stack + 2;

            hash ^= hash_sq_pc(action);
            hash ^= hash_stack(sq, new_stack as _, STACK_CAP as _);
            unsafe {
                // Touch the cache line.
                self.update
                    .tt
                    .add(hash as usize & self.update.tt_idx_mask)
                    .read_volatile();
            }
            self.update.hashes[new_ply] = hash;

            log!(self, match action >> TAG_OFFSET {
                FLAT_TAG => MakePlaceFlat,
                WALL_TAG => MakePlaceWall,
                CAP_TAG => MakePlaceCap,
                _ => unreachable!(),
            });

            let inf = &mut self.update.influence.pair_mut()[player];

            // Unconditionally store. TODO: Is it worth checking if it needs unmaking?
            unmake.val.influence = *inf;

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
            let adjacent = *inf & simd_road_bit;

            // Set the road and noble bits, if appropriate piece.
            let new_road = self.copy.road ^ road_bit;
            new.val.road = new_road;
            new.val.noble = self.copy.noble ^ (action as Bb >> NOBLE_TAG_OFFSET & 1) << sq;

            // Clear and set the owner.
            let new_owner = self.copy.owner & !(1 << sq) | (player as Bb) << sq;
            new.val.owner = new_owner;

            // Placement is irreversible.
            new.val.last_irreversible = self.copy.ply;

            // Set the stack.
            *unsafe { self.update.stacks.get_unchecked_mut(sq as usize) } = new_stack;

            // Copy the tall bitboard unchanged.
            new.val.tall = self.copy.tall;

            // Check if the new road bit is adjacent to any of the influence masks.
            if adjacent.reduce_or() != 0 {
                // Prepare to add the neighbors to the adjacent masks.
                let neighbors = adjacent
                    .simd_ne(Simd::splat(0))
                    .select(simd_road_bit.neighbors(), Simd::splat(0));

                // Add the neighbors.
                let mut old_inf = *inf;
                *inf |= neighbors; // Codegens a spurious store. Alternatives worse.

                // Get a bitboard representing our road pieces.
                let my_road = Simd::splat(new_road & (new_owner ^ (player as Bb).wrapping_sub(1)));

                // Expandable neighbors are those that are our road pieces.
                let mut expandable = neighbors & my_road;

                // Count iterations for logging purposes.
                let mut i = 0;

                // If we have any such neighbors that are not part of the old influence, expand.
                // Assume winning placements are rare and/or handled before make.
                while (expandable & !old_inf).reduce_or() != 0 {
                    i += 1;
                    old_inf = *inf;
                    *inf |= expandable.neighbors();
                    expandable = *inf & my_road;
                }

                log!(self, PlacementExpansionIterations(i));
            }
        }
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
        self.update.influence.pair_mut()[player] = unmake.influence; // this is ass
    }
}
