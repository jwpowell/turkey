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
            // We know that new_id is non-zero because we just incremented it.
            // It won't overflow because it's 64-bit unsigned which will take
            // well beyond the lifetime of the program to reach u64::MAX.

            Self(NonZeroU64::new_unchecked(new_id))
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> + FusedIterator {
        (0..).map(|_| Self::new())
    }
}
