use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// Returns a unique monotonically increasing number.
///
/// This function is thread-safe and can be called from multiple threads concurrently. Each call is
/// guaranteed to return a non-zero unique `u64`.
///
/// # Examples
///
/// ~~~
/// use std::thread;
///
/// use turkey::utils::unique;
///
/// let mut v1 = Vec::new();
/// let mut v2 = Vec::new();
///
/// let t1 = thread::spawn(move || {
///     for _ in 0..1000 {
///         v1.push(unique());
///     }
///     v1
/// });
///
/// let t2 = thread::spawn(move || {
///     for _ in 0..1000 {
///         v2.push(unique());
///     }
///     v2
/// });
///
/// let v1 = t1.join().unwrap();
/// let v2 = t2.join().unwrap();
///
/// let mut all = v1;
/// all.extend(v2);
/// let len = all.len();
/// all.sort_unstable();
/// all.dedup();
/// assert_eq!(all.len(), len);  // no duplicates
/// ~~~
pub fn unique() -> u64 {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
