use crate::*;

const BOTTOM: usize = 0;
const TOP: usize = 1;
const LEFT: usize = 2;
const RIGHT: usize = 3;

#[repr(align(32))]
#[derive(Debug, Clone, Copy)]
pub struct Influence([Bitboard; 4]);

impl Influence {
    pub const EMPTY: Self = {
        let mut edges = [0; 4];

        edges[BOTTOM] = sq(0).row_bitboard();
        edges[TOP] = sq((SIZE - 1) * ROW_LEN).row_bitboard();
        edges[LEFT] = sq(0).col_bitboard();
        edges[RIGHT] = sq(SIZE - 1).col_bitboard();

        Self(edges)
    };

    pub fn new(road: Bitboard, fast: bool) -> (Self, bool) {
        fn expand(edges: &mut [Bitboard; 4], and_with: Bitboard) -> [Bitboard; 4] {
            edges.map(|c| (c | c << 1 | c >> 1 | c << ROW_LEN | c >> ROW_LEN) & and_with)
        }

        assert_ne!(PADDING, 0);

        let mut influence = Self::EMPTY;
        let edges = &mut influence.0;

        edges.iter_mut().for_each(|e| *e &= road);

        let has_road = loop {
            // Fill all nearby road tiles
            let next = expand(edges, road);

            if (next[BOTTOM] & next[TOP] != 0) | (next[LEFT] & next[RIGHT] != 0) {
                // If either pair of edges met, there is a road
                break true;
            }

            if fast {
                // Partial computation
                if ((next[BOTTOM] == edges[BOTTOM]) | (next[TOP] == edges[TOP]))
                    & ((next[LEFT] == edges[LEFT]) | (next[RIGHT] == edges[RIGHT]))
                {
                    // If at least one edge stagnated in both directions, there can be no road
                    break false;
                }
            } else {
                // Full computation
                if (0..4).all(|i| edges[i] == next[i]) {
                    // If all edges stagnated, we're done expanding
                    break false;
                }
            }

            *edges = next;
        };

        // Expand all edges one more time
        // TODO: Consider removing the final AND with BOARD
        // FIXME: edges |= Self::EMPTY
        *edges = expand(edges, BOARD);

        (influence, has_road)
    }
}

impl Default for Influence {
    fn default() -> Self {
        Self::EMPTY
    }
}
