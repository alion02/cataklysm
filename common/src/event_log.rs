#[cfg(feature = "event-log")]
mod inner {
    use core::hash::Hash;

    use hashbrown::HashMap;

    pub struct EventLog<E> {
        skip: u32,
        events: HashMap<E, u64>,
    }

    impl<E> EventLog<E> {
        #[inline]
        pub fn should_log(&mut self) -> bool {
            self.skip -= 1;
            self.skip == 0
        }
    }

    impl<E: Hash + Eq> EventLog<E> {
        #[cold]
        #[inline(never)]
        pub fn log(&mut self, event: E) {
            *self.events.entry(event).or_default() += 1;
        }
    }
}

#[cfg(not(feature = "event-log"))]
mod inner {
    use core::marker::PhantomData;

    pub struct EventLog<E>(PhantomData<E>);

    impl<E> EventLog<E> {
        #[inline]
        pub fn should_log(&mut self) -> bool {
            false
        }

        #[cold]
        #[inline(never)]
        pub fn log(&mut self, event: E) {}
    }
}

pub use inner::*;
