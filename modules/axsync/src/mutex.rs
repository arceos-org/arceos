//! A blocking mutex implementation.

use core::sync::atomic::{AtomicU64, Ordering};

use axtask::{WaitQueue, current};

/// A [`lock_api::RawMutex`] implementation.
///
/// When the mutex is locked, the current task will block and be put into the
/// wait queue. When the mutex is unlocked, ownership is handed off to at most
/// one task waiting on the queue; if no tasks are waiting, the mutex simply
/// becomes unlocked.
pub struct RawMutex {
    wq: WaitQueue,
    owner_id: AtomicU64,
}

impl RawMutex {
    /// Creates a [`RawMutex`].
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            wq: WaitQueue::new(),
            owner_id: AtomicU64::new(0),
        }
    }

    #[inline(always)]
    fn is_owner(&self, owner_id: u64) -> bool {
        self.owner_id.load(Ordering::Acquire) == owner_id
    }
}

impl Default for RawMutex {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl lock_api::RawMutex for RawMutex {
    type GuardMarker = lock_api::GuardSend;

    /// Initial value for an unlocked mutex.
    ///
    /// A “non-constant” const item is a legacy way to supply an initialized
    /// value to downstream static items. Can hopefully be replaced with
    /// `const fn new() -> Self` at some point.
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = RawMutex::new();

    #[inline(always)]
    fn lock(&self) {
        let current_id = current().id().as_u64();

        loop {
            // Can fail to lock even if the spinlock is not locked. May be more efficient than `try_lock`
            // when called in a loop.
            match self.owner_id.compare_exchange_weak(
                0,
                current_id,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(owner_id) => {
                    assert_ne!(
                        owner_id, current_id,
                        "Thread({current_id}) tried to acquire mutex it already owns.",
                    );
                    // Wait until someone hands off lock to me or lock is released
                    self.wq
                        .wait_until(|| self.is_owner(current_id) || !self.is_locked());
                    // This check is necessary: some newcomers may race with a wakened one.
                    if self.is_owner(current_id) {
                        break;
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn try_lock(&self) -> bool {
        let current_id = current().id().as_u64();
        // The reason for using a strong compare_exchange is explained here:
        // https://github.com/Amanieu/parking_lot/pull/207#issuecomment-575869107
        self.owner_id
            .compare_exchange(0, current_id, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    #[inline(always)]
    unsafe fn unlock(&self) {
        let owner_id = self.owner_id.load(Ordering::Acquire);
        let current_id = current().id().as_u64();
        assert_eq!(
            owner_id, current_id,
            "Thread({current_id}) tried to release mutex it doesn't own",
        );
        // wake up one waiting thread.
        self.wq.notify_one_with(true, |id: u64| {
            self.owner_id.swap(id, Ordering::Release);
        });
    }

    #[inline(always)]
    fn is_locked(&self) -> bool {
        self.owner_id.load(Ordering::Acquire) != 0
    }
}

/// An alias of [`lock_api::Mutex`].
pub type Mutex<T> = lock_api::Mutex<RawMutex, T>;
/// An alias of [`lock_api::MutexGuard`].
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawMutex, T>;

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use axtask as thread;

    use crate::Mutex;

    static INIT: Once = Once::new();

    fn may_interrupt() {
        // simulate interrupts
        if fastrand::u8(0..3) == 0 {
            thread::yield_now();
        }
    }

    #[test]
    fn lots_and_lots() {
        INIT.call_once(thread::init_scheduler);

        const NUM_TASKS: u32 = 10;
        const NUM_ITERS: u32 = 10_000;
        static M: Mutex<u32> = Mutex::new(0);

        fn inc(delta: u32) {
            for _ in 0..NUM_ITERS {
                let mut val = M.lock();
                *val += delta;
                may_interrupt();
                drop(val);
                may_interrupt();
            }
        }

        for _ in 0..NUM_TASKS {
            thread::spawn(|| inc(1));
            thread::spawn(|| inc(2));
        }

        println!("spawn OK");
        loop {
            let val = M.lock();
            if *val == NUM_ITERS * NUM_TASKS * 3 {
                break;
            }
            may_interrupt();
            drop(val);
            may_interrupt();
        }

        assert_eq!(*M.lock(), NUM_ITERS * NUM_TASKS * 3);
        println!("Mutex test OK");
    }
}
