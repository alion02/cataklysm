#[cfg(feature = "event-log")]
mod inner {
    use crate::Event;
    use alloc::boxed::Box;
    use hashbrown::HashMap;
    use rand_chacha::ChaCha20Rng;
    use rand_distr::{Distribution, Geometric};

    pub struct EventLog {
        skip: i32,
        inner: Box<EventLogInner>,
    }

    struct EventLogInner {
        events: HashMap<Event, u64>,
        dist: Geometric,
        rng: ChaCha20Rng,
    }

    impl EventLog {
        #[inline]
        pub fn try_log(&mut self, weight: i32) -> bool {
            self.skip -= weight;
            self.skip < 0
        }

        #[cold]
        #[inline(never)]
        pub fn log(&mut self, event: Event) {
            let entry = self.inner.events.entry(event).or_default();
            while self.skip < 0 {
                *entry += 1;
                self.skip += self.inner.dist.sample(&mut self.inner.rng) as i32 + 1;
            }
        }
    }
}

#[cfg(not(feature = "event-log"))]
mod inner {
    use crate::Event;

    pub struct EventLog;

    impl EventLog {
        #[inline]
        pub fn should_log(&mut self) -> bool {
            false
        }

        pub fn log(&mut self, _: Event) {}
    }
}

pub use inner::*;

#[derive(Hash, PartialEq, Eq)]
pub struct Event {
    pub ply: u16,
    pub kind: EventKind,
}

#[derive(Hash, PartialEq, Eq)]
pub enum EventKind {
    GenPlaceFlat,
    GenPlaceWall,
    GenPlaceCap,
    MakePlaceFlat,
    MakePlaceWall,
    MakePlaceCap,
    PlacementExpansionIterations(u8),
}
