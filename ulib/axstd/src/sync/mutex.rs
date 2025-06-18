//! A naïve sleeping mutex.

use core::sync::atomic::{AtomicU64, Ordering};

use arceos_api::task::{self as api, AxWaitQueueHandle};

/// A [`lock_api::RawMutex`] implementation.
///
/// When the mutex is locked, the current task will block and be put into the
/// wait queue. When the mutex is unlocked, all tasks waiting on the queue
/// will be woken up.
pub struct RawMutex {
    wq: AxWaitQueueHandle,
    owner_id: AtomicU64,
}

impl RawMutex {
    /// Creates a [`RawMutex`].
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            wq: AxWaitQueueHandle::new(),
            owner_id: AtomicU64::new(0),
        }
    }
}

unsafe impl lock_api::RawMutex for RawMutex {
    const INIT: Self = RawMutex::new();

    type GuardMarker = lock_api::GuardSend;

    #[inline(always)]
    fn lock(&self) {
        let current_id = api::ax_current_task_id();
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
                    // Wait until the lock looks unlocked before retrying
                    api::ax_wait_queue_wait_until(&self.wq, || !self.is_locked(), None);
                }
            }
        }
    }

    #[inline(always)]
    fn try_lock(&self) -> bool {
        let current_id = api::ax_current_task_id();
        // The reason for using a strong compare_exchange is explained here:
        // https://github.com/Amanieu/parking_lot/pull/207#issuecomment-575869107
        self.owner_id
            .compare_exchange(0, current_id, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    #[inline(always)]
    unsafe fn unlock(&self) {
        let owner_id = self.owner_id.swap(0, Ordering::Release);
        let current_id = api::ax_current_task_id();
        assert_eq!(
            owner_id, current_id,
            "Thread({current_id}) tried to release mutex it doesn't own",
        );
        // wake up one waiting thread.
        api::ax_wait_queue_wake(&self.wq, 1);
    }

    #[inline(always)]
    fn is_locked(&self) -> bool {
        self.owner_id.load(Ordering::Relaxed) != 0
    }
}

/// An alias of [`lock_api::Mutex`].
pub type Mutex<T> = lock_api::Mutex<RawMutex, T>;
/// An alias of [`lock_api::MutexGuard`].
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawMutex, T>;
