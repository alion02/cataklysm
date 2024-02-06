use crate::*;

const BOTTOM: usize = 0;
const TOP: usize = 1;
const LEFT: usize = 2;
const RIGHT: usize = 3;

#[repr(align(32))]
#[derive(Debug, Clone, Copy)]
pub struct Influence([Bitboard; 4]);

impl Influence {
    pub const EDGES: Self = {
        let mut edges = [0; 4];

        edges[BOTTOM] = sq(0).row_bitboard();
        edges[TOP] = sq((SIZE - 1) * ROW_LEN).row_bitboard();
        edges[LEFT] = sq(0).col_bitboard();
        edges[RIGHT] = sq(SIZE - 1).col_bitboard();

        Self(edges)
    };

    pub fn new(road: Bitboard, fast: bool) -> (Self, bool) {
        let mut influence = Self::EDGES & road;

        let has_road = loop {
            // Fill all nearby road tiles
            let next = influence | influence.spread() & road;

            // TODO: Use intersections_of_opposites?
            if (next[BOTTOM] & next[TOP] != 0) | (next[LEFT] & next[RIGHT] != 0) {
                // If either pair of edges met, there is a road
                break true;
            }

            if fast {
                // Partial computation
                if ((next[BOTTOM] == influence[BOTTOM]) | (next[TOP] == influence[TOP]))
                    & ((next[LEFT] == influence[LEFT]) | (next[RIGHT] == influence[RIGHT]))
                {
                    // If at least one edge stagnated in both directions, there can be no road
                    break false;
                }
            } else {
                // Full computation
                // TODO: Use ==?
                if (0..4).all(|i| influence[i] == next[i]) {
                    // If all edges stagnated, we're done expanding
                    break false;
                }
            }

            influence = next;
        };

        // Expand all edges one more time
        // TODO: Consider removing the final AND with BOARD
        influence |= Self::EDGES | influence.spread() & BOARD;

        (influence, has_road)
    }

    pub fn spread(self) -> Self {
        Self(self.0.map(Bitboard::spread))
    }

    pub fn intersections_of_opposites(self) -> Bitboard {
        self[BOTTOM] & self[TOP] | self[LEFT] & self[RIGHT]
    }
}

impl BitAnd<Bitboard> for Influence {
    type Output = Influence;

    fn bitand(self, rhs: Bitboard) -> Self::Output {
        Self(self.0.map(|lhs| lhs & rhs))
    }
}

impl BitAndAssign<Bitboard> for Influence {
    fn bitand_assign(&mut self, rhs: Bitboard) {
        for lhs in &mut self.0 {
            *lhs &= rhs;
        }
    }
}

impl BitOr<Bitboard> for Influence {
    type Output = Influence;

    fn bitor(self, rhs: Bitboard) -> Self::Output {
        Self(self.0.map(|lhs| lhs | rhs))
    }
}

impl BitOrAssign<Bitboard> for Influence {
    fn bitor_assign(&mut self, rhs: Bitboard) {
        for lhs in &mut self.0 {
            *lhs |= rhs;
        }
    }
}

impl BitXor<Bitboard> for Influence {
    type Output = Influence;

    fn bitxor(self, rhs: Bitboard) -> Self::Output {
        Self(self.0.map(|lhs| lhs ^ rhs))
    }
}

impl BitXorAssign<Bitboard> for Influence {
    fn bitxor_assign(&mut self, rhs: Bitboard) {
        for lhs in &mut self.0 {
            *lhs ^= rhs;
        }
    }
}

impl BitAnd for Influence {
    type Output = Influence;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(make_arr(|i| self[i] & rhs[i]))
    }
}

impl BitAndAssign for Influence {
    fn bitand_assign(&mut self, rhs: Self) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0) {
            *lhs &= rhs;
        }
    }
}

impl BitOr for Influence {
    type Output = Influence;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(make_arr(|i| self[i] | rhs[i]))
    }
}

impl BitOrAssign for Influence {
    fn bitor_assign(&mut self, rhs: Self) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0) {
            *lhs |= rhs;
        }
    }
}

impl BitXor for Influence {
    type Output = Influence;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(make_arr(|i| self[i] ^ rhs[i]))
    }
}

impl BitXorAssign for Influence {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0) {
            *lhs ^= rhs;
        }
    }
}

impl Index<usize> for Influence {
    type Output = Bitboard;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Influence {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
