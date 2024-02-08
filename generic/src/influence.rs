use crate::*;

pub const BOTTOM: usize = 0;
pub const TOP: usize = 1;
pub const LEFT: usize = 2;
pub const RIGHT: usize = 3;

#[repr(align(32))]
#[derive(Debug, Clone, Copy)]
pub struct Influence([Bitboard; 4]);

impl Influence {
    pub const EDGES: Self = {
        let mut edges = [0; 4];

        edges[BOTTOM] = EDGE_BOTTOM;
        edges[TOP] = EDGE_TOP;
        edges[LEFT] = EDGE_LEFT;
        edges[RIGHT] = EDGE_RIGHT;

        Self(edges)
    };

    #[inline(always)]
    pub fn clear_and_flood(&mut self, road: Bitboard, fast: bool) -> bool {
        self.clear();
        self.flood(road, fast)
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        *self = Self::EDGES;
    }

    #[inline(always)]
    pub fn flood(&mut self, road: Bitboard, fast: bool) -> bool {
        *self &= road;

        let has_road = loop {
            // Fill all nearby road tiles
            let next = *self | self.spread() & road;

            if next.intersections_of_opposites() != 0 {
                // If either pair of edges met, there is a road
                break true;
            }

            if fast {
                // Partial computation
                if ((next[BOTTOM] == self[BOTTOM]) | (next[TOP] == self[TOP]))
                    & ((next[LEFT] == self[LEFT]) | (next[RIGHT] == self[RIGHT]))
                {
                    // If at least one edge stagnated in both directions, there can be no road
                    break false;
                }
            } else {
                // Full computation
                if &next == self {
                    // If all edges stagnated, we're done expanding
                    break false;
                }
            }

            *self = next;
        };

        // Expand all edges one more time
        // TODO: Consider removing the final AND with BOARD
        *self |= Self::EDGES | self.spread() & BOARD;

        has_road
    }

    // pub fn extend(&mut self, new_road: Bitboard) {
    //     let point = new_road.spread() & BOARD;
    //     *self |= Self(self.0.map(|b| if b & new_road != 0 { point } else { 0 }));
    // }

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

impl PartialEq for Influence {
    fn eq(&self, other: &Self) -> bool {
        (0..4).all(|i| self[i] == other[i])
    }
}

impl Eq for Influence {}
