use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Action(ActionBacking);

impl Action {
    const TYPE_OFFSET: u32 = (ARR_LEN - 1).ilog2() + 1;
    const PAT_OFFSET: u32 = Self::TYPE_OFFSET + 2;

    pub const PASS: Self = Self(0);

    pub fn place(sq: Square, piece: Piece) -> Self {
        Self(sq.0 as ActionBacking | (piece as ActionBacking) << Self::TYPE_OFFSET)
    }

    pub fn spread(sq: Square, dir: Direction, pat: Pattern) -> Self {
        Self(
            sq.0 as ActionBacking
                | (dir as ActionBacking) << Self::TYPE_OFFSET
                | (pat.0 as ActionBacking) << Self::PAT_OFFSET,
        )
    }

    // `self.0 as u32` is unnecessary iff Action is backed by u32
    #[allow(clippy::unnecessary_cast)]
    pub fn branch<S, R>(
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

impl Default for Action {
    fn default() -> Self {
        Self::PASS
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

impl Move for Action {
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pattern(u32);

pub fn pat(pat: u32) -> Pattern {
    debug_assert!(pat > 0);
    debug_assert!(pat < 1 << HAND);

    Pattern(pat)
}

impl Pattern {
    pub fn execute(self) -> (u32, DropCounts) {
        let mut dc = DropCounts(self.0 | 1 << HAND);
        // TODO: Investigate unwrap
        (HAND - dc.next().unwrap(), dc)
    }
}
