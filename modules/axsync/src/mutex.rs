//! A naïve sleeping mutex.

use core::sync::atomic::{AtomicU64, Ordering};

use axtask::{current, future::block_on, yield_now};
use event_listener::{Event, listener};

/// A [`lock_api::RawMutex`] implementation.
///
/// When the mutex is locked, the current task will block and be put into the
/// wait queue. When the mutex is unlocked, all tasks waiting on the queue
/// will be woken up.
pub struct RawMutex {
    event: Event,
    owner_id: AtomicU64,
}

impl RawMutex {
    /// Creates a [`RawMutex`].
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            event: Event::new(),
            owner_id: AtomicU64::new(0),
        }
    }
}

struct Spin(u32);

impl Spin {
    #[inline]
    fn spin(&mut self) -> bool {
        if self.0 >= 10 {
            return false;
        }
        self.0 += 1;
        if self.0 <= 3 {
            for _ in 0..(1 << self.0) {
                core::hint::spin_loop();
            }
        } else {
            yield_now();
        }
        true
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
        let mut spin = Spin(0);
        let mut owner_id = self.owner_id.load(Ordering::Relaxed);

        loop {
            assert_ne!(
                owner_id,
                current_id,
                "{} tried to acquire mutex it already owns.",
                current().id_name()
            );

            if owner_id == 0 {
                match self.owner_id.compare_exchange_weak(
                    owner_id,
                    current_id,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => owner_id = x,
                }
                continue;
            }

            if spin.spin() {
                owner_id = self.owner_id.load(Ordering::Relaxed);
                continue;
            }

            listener!(self.event => listener);

            owner_id = self.owner_id.load(Ordering::Relaxed);
            if owner_id == 0 {
                continue;
            }

            block_on(listener);
            owner_id = self.owner_id.load(Ordering::Relaxed);
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
        let owner_id = self.owner_id.swap(0, Ordering::Release);
        assert_eq!(
            owner_id,
            current().id().as_u64(),
            "{} tried to release mutex it doesn't own",
            current().id_name()
        );
        self.event.notify(1);
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

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use axtask as thread;

    use crate::Mutex;

    static INIT: Once = Once::new();

    fn may_interrupt() {
        // simulate interrupts
        if rand::random::<u32>() % 3 == 0 {
            thread::yield_now();
        }
    }

    #[test]
    fn lots_and_lots() {
        INIT.call_once(thread::init_scheduler);

        const NUM_TASKS: u32 = 10;
        const NUM_ITERS: u32 = 10_000;
        static M: Mutex<u32> = Mutex::new(0);

        fn inc(delta: u32) -> ! {
            for _ in 0..NUM_ITERS {
                let mut val = M.lock();
                *val += delta;
                may_interrupt();
                drop(val);
                may_interrupt();
            }
        }

        for _ in 0..NUM_TASKS {
            thread::spawn(|| inc(1), "".into());
            thread::spawn(|| inc(2), "".into());
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
