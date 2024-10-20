//! Synchronization primitive allowing multiple threads to synchronize the
//! beginning of some computation.
//!
//! Implementation adapted from the 'Barrier' type of the standard library. See:
//! <https://doc.rust-lang.org/std/sync/struct.Barrier.html>
use core::fmt;

use crate::condvar::Condvar;
use crate::mutex::Mutex;

/// A barrier enables multiple threads to synchronize the beginning
/// of some computation.
pub struct Barrier {
    lock: Mutex<BarrierState>,
    cvar: Condvar,
    num_threads: usize,
}

// The inner state of a double barrier
struct BarrierState {
    count: usize,
    generation_id: usize,
}

/// A `BarrierWaitResult` is returned by [`Barrier::wait()`] when all threads
/// in the [`Barrier`] have rendezvoused.
pub struct BarrierWaitResult(bool);

impl fmt::Debug for Barrier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Barrier").finish_non_exhaustive()
    }
}

impl Barrier {
    /// Creates a new barrier that can block a given number of threads.
    ///
    /// A barrier will block `n`-1 threads which call [`wait()`] and then wake
    /// up all threads at once when the `n`th thread calls [`wait()`].
    ///
    /// [`wait()`]: Barrier::wait
    pub const fn new(n: usize) -> Self {
        Self {
            lock: Mutex::new(BarrierState {
                count: 0,
                generation_id: 0,
            }),
            cvar: Condvar::new(),
            num_threads: n,
        }
    }

    /// Blocks the current thread until all threads have rendezvoused here.
    ///
    /// Barriers are re-usable after all threads have rendezvoused once, and can
    /// be used continuously.
    ///
    /// A single (arbitrary) thread will receive a [`BarrierWaitResult`] that
    /// returns `true` from [`BarrierWaitResult::is_leader()`] when returning
    /// from this function, and all other threads will receive a result that
    /// will return `false` from [`BarrierWaitResult::is_leader()`].
    pub fn wait(&self) -> BarrierWaitResult {
        let mut lock = self.lock.lock();
        lock.count += 1;

        if lock.count < self.num_threads {
            // not the leader
            let local_gen = lock.generation_id;
            let _guard = self
                .cvar
                .wait_while(lock, |state| local_gen == state.generation_id);
            BarrierWaitResult(false)
        } else {
            // this thread is the leader,
            //   and is responsible for incrementing the generation
            lock.count = 0;
            lock.generation_id = lock.generation_id.wrapping_add(1);
            self.cvar.notify_all();
            BarrierWaitResult(true)
        }
    }
}

impl fmt::Debug for BarrierWaitResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BarrierWaitResult")
            .field("is_leader", &self.is_leader())
            .finish()
    }
}

impl BarrierWaitResult {
    /// Returns whether this thread from [`wait`] is the "leader thread".
    ///
    /// Only one thread will have `true` returned from their result, all other
    /// threads will have `false` returned.
    ///
    /// [`wait`]: struct.Barrier.html#method.wait
    ///
    /// # Examples
    ///
    /// ```
    /// use spin;
    ///
    /// let barrier = spin::Barrier::new(1);
    /// let barrier_wait_result = barrier.wait();
    /// println!("{:?}", barrier_wait_result.is_leader());
    /// ```
    pub fn is_leader(&self) -> bool {
        self.0
    }
}
