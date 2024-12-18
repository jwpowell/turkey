use std::iter::FusedIterator;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Unique(NonZeroU64);

impl Default for Unique {
    fn default() -> Self {
        Self::new()
    }
}

static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(1);

impl Unique {
    pub fn new() -> Self {
        let new_id = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);

        unsafe {
            // `new_id` starts at 1. It is never decremented directly. So, the only way it could be
            // 0 is if the `fetch_add` above overflowed. That would happen after (2^64)-1
            // increments.
            //
            // If we increment once a cycle on 256 threads on a 6GHz processor, which is an
            // extremely generous estimate, it would take ((1/6GHz) * (2^64) - 1 seconds)/(256
            // threads) = 4.5 months.
            //
            // ASSUMPTION: This program will not be running for more than 4.5 months.
            // ASSUMPTION: This program will not be running on a system as powerful as this.
            //
            // Under these assumptions, `new_id` will never  be 0.

            Self(NonZeroU64::new_unchecked(new_id))
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> + FusedIterator {
        (0..).map(|_| Self::new())
    }
}
