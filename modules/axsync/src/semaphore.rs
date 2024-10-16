//! A counting, blocking, semaphore.
//! Implementation adapted from the **unstable** 'Semaphore' type of the standard library.
//! See: <https://doc.rust-lang.org/1.8.0/std/sync/struct.Semaphore.html>
//! In fact, it is marked as **Deprecated** since 1.7.0:
//! due to: easily confused with system semaphores and not used enough to pull its weight.
//!
//! Note: [`Semaphore`] is not available when the `multitask` feature is disabled.

use crate::{Condvar, Mutex};

/// A counting, blocking, semaphore.
///
/// Semaphores are a form of atomic counter where access is only granted if the
/// counter is a positive value. Each acquisition will block the calling thread
/// until the counter is positive, and each release will increment the counter
/// and unblock any threads if necessary.
pub struct Semaphore {
    lock: Mutex<isize>,
    cvar: Condvar,
}

/// An RAII guard which will release a resource acquired from a semaphore when
/// dropped.
pub struct SemaphoreGuard<'a> {
    sem: &'a Semaphore,
}

impl Semaphore {
    /// Creates a new semaphore with the initial count specified.
    ///
    /// The count specified can be thought of as a number of resources, and a
    /// call to `acquire` or `access` will block until at least one resource is
    /// available. It is valid to initialize a semaphore with a negative count.
    pub fn new(count: isize) -> Semaphore {
        Semaphore {
            lock: Mutex::new(count),
            cvar: Condvar::new(),
        }
    }

    /// Acquires a resource of this semaphore, blocking the current thread until
    /// it can do so.
    ///
    /// This method will block until the internal count of the semaphore is at
    /// least 1.
    pub fn acquire(&self) {
        let mut count = self.lock.lock();
        while *count <= 0 {
            let count_res = self.cvar.wait(count);
            count = count_res;
        }
        *count -= 1;
    }

    /// Release a resource from this semaphore.
    ///
    /// This will increment the number of resources in this semaphore by 1 and
    /// will notify any pending waiters in `acquire` or `access` if necessary.
    pub fn release(&self) {
        *self.lock.lock() += 1;
        self.cvar.notify_one();
    }

    /// Acquires a resource of this semaphore, returning an RAII guard to
    /// release the semaphore when dropped.
    ///
    /// This function is semantically equivalent to an `acquire` followed by a
    /// `release` when the guard returned is dropped.
    pub fn access(&self) -> SemaphoreGuard {
        self.acquire();
        SemaphoreGuard { sem: self }
    }
}

impl<'a> Drop for SemaphoreGuard<'a> {
    fn drop(&mut self) {
        self.sem.release();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::sync::Once;

    use axtask as thread;

    use crate::Semaphore;

    static INIT: Once = Once::new();

    #[test]
    fn test_sem_acquire_release() {
        INIT.call_once(thread::init_scheduler);

        let s = Semaphore::new(1);
        s.acquire();
        s.release();
        s.acquire();
    }

    #[test]
    fn test_sem_basic() {
        INIT.call_once(thread::init_scheduler);

        let s = Semaphore::new(1);
        let _g = s.access();
    }

    #[test]
    fn test_sem_as_mutex() {
        INIT.call_once(thread::init_scheduler);

        let s = Arc::new(Semaphore::new(1));
        let s2 = s.clone();
        let _t = thread::spawn(move || {
            let _g = s2.access();
        });
        let _g = s.access();
    }

    #[test]
    fn test_sem_as_cvar() {
        INIT.call_once(thread::init_scheduler);

        // Child waits and parent signals
        let (tx, rx) = channel();
        let s = Arc::new(Semaphore::new(0));
        let s2 = s.clone();
        let _t = thread::spawn(move || {
            s2.acquire();
            tx.send(()).unwrap();
        });
        s.release();
        thread::yield_now();
        let _ = rx.recv();

        // Parent waits and child signals
        let (tx, rx) = channel();
        let s = Arc::new(Semaphore::new(0));
        let s2 = s.clone();
        let _t = thread::spawn(move || {
            s2.release();
            thread::yield_now();
            let _ = rx.recv();
        });
        s.acquire();
        tx.send(()).unwrap();
        thread::yield_now();
    }

    #[test]
    fn test_sem_multi_resource() {
        INIT.call_once(thread::init_scheduler);

        // Parent and child both get in the critical section at the same
        // time, and shake hands.
        let s = Arc::new(Semaphore::new(2));
        let s2 = s.clone();
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        let _t = thread::spawn(move || {
            let _g = s2.access();
            thread::yield_now();
            let _ = rx2.recv();
            tx1.send(()).unwrap();
        });
        let _g = s.access();
        thread::yield_now();
        tx2.send(()).unwrap();
        thread::yield_now();
        rx1.recv().unwrap();
    }

    #[test]
    fn test_sem_runtime_friendly_blocking() {
        INIT.call_once(thread::init_scheduler);

        let s = Arc::new(Semaphore::new(1));
        let s2 = s.clone();
        let (tx, rx) = channel();
        {
            let _g = s.access();
            thread::spawn(move || {
                tx.send(()).unwrap();
                thread::yield_now();
                drop(s2.access());
                tx.send(()).unwrap();
                thread::yield_now();
            });
            thread::yield_now();
            rx.recv().unwrap(); // wait for child to come alive
        }
        thread::yield_now();
        rx.recv().unwrap(); // wait for child to be done
    }
}
