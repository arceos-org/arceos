use axtask as thread;

use crate::Barrier;

const NUM_TASKS: u32 = 10;
const NUM_ITERS: u32 = 10_000;

#[test]
fn test_barrier() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    static BARRIER: Barrier = Barrier::new(NUM_TASKS as usize);

    let mut join_handlers = Vec::new();

    fn rendezvous() {
        for _ in 0..NUM_ITERS {
            BARRIER.wait();
        }
    }

    for _ in 0..NUM_TASKS {
        join_handlers.push(thread::spawn(rendezvous));
    }

    // Wait for all threads to finish.
    for join_handler in join_handlers {
        join_handler.join();
    }

    println!("Barrier test OK");
}

#[test]
fn test_wait_result() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    static BARRIER: Barrier = Barrier::new(1);

    // The first thread to call `wait` will be the leader.
    assert_eq!(BARRIER.wait().is_leader(), true);

    // Since the barrier is reusable, the next thread to call `wait` will also be the leader.
    assert_eq!(BARRIER.wait().is_leader(), true);

    static BARRIER2: Barrier = Barrier::new(2);

    thread::spawn(|| {
        assert_eq!(BARRIER2.wait().is_leader(), true);
    });

    // The first thread to call `wait` won't be the leader.
    assert_eq!(BARRIER2.wait().is_leader(), false);

    thread::yield_now();

    println!("BarrierWaitResult test OK");
}

#[test]
fn test_barrier_wait_result() {
    use std::sync::mpsc::{channel, TryRecvError};
    use std::sync::Arc;

    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let barrier = Arc::new(Barrier::new(NUM_TASKS as _));
    let (tx, rx) = channel();

    let mut join_handlers = Vec::new();

    for _ in 0..NUM_TASKS - 1 {
        let c = barrier.clone();
        let tx = tx.clone();
        join_handlers.push(thread::spawn(move || {
            tx.send(c.wait().is_leader()).unwrap();
        }));
    }

    // At this point, all spawned threads should be blocked,
    // so we shouldn't get anything from the port
    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));

    let mut leader_found = barrier.wait().is_leader();

    // Wait for all threads to finish.
    for join_handler in join_handlers {
        join_handler.join();
    }

    // Now, the barrier is cleared and we should get data.
    for _ in 0..NUM_TASKS - 1 {
        if rx.recv().unwrap() {
            assert!(!leader_found);
            leader_found = true;
        }
    }
    assert!(leader_found);
}
