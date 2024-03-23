use crate::*;

impl State {
    #[no_mangle]
    #[inline]
    pub fn player(&self) -> bool {
        self.ply & 1 != 0
    }

    #[inline]
    pub fn piece(&self) -> Bb {
        self.road | self.noble
    }

    #[inline]
    pub fn for_actions<B, C>(
        &mut self,
        mut acc: C,
        f: fn(&mut Self, C, u16) -> ControlFlow<B, C>,
    ) -> ControlFlow<B, C> {
        let empty = self.piece() ^ BOARD;
        macro_rules! for_placements {
            ($pc:expr) => {
                let mut left = empty;
                // Assumption: actions shall not be generated on completed games.
                loop {
                    acc = f(self, acc, left.trailing_zeros() as u16 | $pc << 6)?;
                    left &= left - 1;
                    if left == 0 {
                        break;
                    }
                }
            };
        }

        // Only do stone placements if we have any stones in reserve. We're technically checking
        // the wrong reserves in the opening because we assume this passes anyway.
        if self.stones_left[self.player()] > 0 {
            // Flat placements.
            for_placements!(1);

            // If we're in the opening, skip over all the other kinds of actions. This check gets
            // skipped if we don't have stones, but we again don't care.
            if self.ply < 2 {
                return Continue(acc);
            }

            // Wall placements.
            for_placements!(2);
        }

        if self.caps_left[self.player()] > 0 {
            // Cap placements.
            for_placements!(3);
        }

        Continue(acc)
    }

    #[no_mangle]
    #[inline]
    pub fn make(&mut self, unmake: &mut Unmake, action: u16) {
        let player = self.player();
        self.ply += 1;
        if action <= 255 {
            // Check if it's the opening (ply 1 or 2, after the update).
            let player = player ^ (self.ply < 3);
            let sq = action & 0x3F;
            let new_road = (action as Bb >> 6 & 1) << sq;
            let inf = &mut self.influence.pair_mut()[player];
            let simd_new_road = Simd::splat(new_road);
            let adjacent = *inf & simd_new_road;

            // Remove the appropriate piece from the reserves.
            (if action >= 3 << 6 {
                &mut self.caps_left
            } else {
                &mut self.stones_left
            })[player] -= 1;

            // Set the road and noble bits, if appropriate piece.
            self.road ^= new_road;
            self.noble ^= (action as Bb >> 7 & 1) << sq;

            // Clear and set the owner.
            self.owner &= !(1 << sq);
            self.owner |= (player as Bb) << sq;

            // Placement is irreversible.
            self.last_irreversible = self.ply;

            // Set the stack.
            *unsafe { self.stacks.get_unchecked_mut(sq as usize) } = player as Stack | 2;

            // Check if the new road bit is adjacent to any of the influence masks.
            if adjacent.reduce_or() != 0 {
                // Prepare to add the neighbors to the adjacent masks.
                let neighbors = adjacent
                    .simd_ne(Simd::splat(0))
                    .select(simd_new_road.neighbors(), Simd::splat(0));

                // Add the neighbors.
                let mut old_inf = *inf;
                *inf |= neighbors; // Codegens a spurious store. Alternatives worse.

                // Get a bitboard representing our road pieces.
                let my_road =
                    Simd::splat(self.road & (self.owner ^ (player as Bb).wrapping_sub(1)));

                // Expandable neighbors are those that are our road pieces.
                let mut expandable = neighbors & my_road;

                // If we have any such neighbors that are not part of the old influence, expand.
                // Assumption: winning placements are rare & should be handled before make.
                while (expandable & !old_inf).reduce_or() != 0 {
                    old_inf = *inf;
                    *inf |= expandable.neighbors();
                    expandable = *inf & my_road;
                }
            }
        }
    }
}

union Unmake {
    uninit: (),
    place: UnmakePlace,
    spread: UnmakeSpread,
}

#[derive(Clone, Copy)]
struct UnmakePlace;

#[derive(Clone, Copy)]
struct UnmakeSpread;
