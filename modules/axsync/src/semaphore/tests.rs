use std::sync::mpsc::channel;
use std::sync::Arc;

use axtask as thread;

use crate::Semaphore;

#[test]
fn test_sem_acquire_release() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let s = Semaphore::new(1);
    s.acquire();
    s.release();
    s.acquire();
}

#[test]
fn test_sem_basic() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let s = Semaphore::new(1);
    let _g = s.access();
}

#[test]
fn test_sem_as_mutex() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let s = Arc::new(Semaphore::new(1));
    let s2 = s.clone();
    let _t = thread::spawn(move || {
        let _g = s2.access();
    });
    let _g = s.access();
}

#[test]
fn test_sem_as_cvar() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

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
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

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
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

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
