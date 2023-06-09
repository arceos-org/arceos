use std::cell::Cell;
use std::mem::MaybeUninit;
use std::sync::{Mutex, MutexGuard};

static GLOBAL_MUTEX: Mutex<()> = Mutex::new(());

// This is initialized if a thread has acquired the CS, uninitialized otherwise.
static mut GLOBAL_GUARD: MaybeUninit<MutexGuard<'static, ()>> = MaybeUninit::uninit();

std::thread_local!(static IS_LOCKED: Cell<bool> = Cell::new(false));

struct StdCriticalSection;
crate::set_impl!(StdCriticalSection);

unsafe impl crate::Impl for StdCriticalSection {
    unsafe fn acquire() -> bool {
        // Allow reentrancy by checking thread local state
        IS_LOCKED.with(|l| {
            if l.get() {
                // CS already acquired in the current thread.
                return true;
            }

            // Note: it is fine to set this flag *before* acquiring the mutex because it's thread local.
            // No other thread can see its value, there's no potential for races.
            // This way, we hold the mutex for slightly less time.
            l.set(true);

            // Not acquired in the current thread, acquire it.
            let guard = match GLOBAL_MUTEX.lock() {
                Ok(guard) => guard,
                Err(err) => {
                    // Ignore poison on the global mutex in case a panic occurred
                    // while the mutex was held.
                    err.into_inner()
                }
            };
            GLOBAL_GUARD.write(guard);

            false
        })
    }

    unsafe fn release(nested_cs: bool) {
        if !nested_cs {
            // SAFETY: As per the acquire/release safety contract, release can only be called
            // if the critical section is acquired in the current thread,
            // in which case we know the GLOBAL_GUARD is initialized.
            GLOBAL_GUARD.assume_init_drop();

            // Note: it is fine to clear this flag *after* releasing the mutex because it's thread local.
            // No other thread can see its value, there's no potential for races.
            // This way, we hold the mutex for slightly less time.
            IS_LOCKED.with(|l| l.set(false));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate as critical_section;

    #[cfg(feature = "std")]
    #[test]
    #[should_panic(expected = "Not a PoisonError!")]
    fn reusable_after_panic() {
        let _ = thread::spawn(|| {
            critical_section::with(|_| {
                panic!("Boom!");
            })
        })
        .join();

        critical_section::with(|_| {
            panic!("Not a PoisonError!");
        })
    }
}
