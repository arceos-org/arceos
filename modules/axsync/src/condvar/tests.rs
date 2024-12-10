use std::sync::{mpsc::channel, Arc};

use axtask as thread;

use crate::{Condvar, Mutex};

const INIT_VALUE: u32 = 0;
const NUM_TASKS: u32 = 10;
const NUM_ITERS: u32 = 10_000;

fn may_interrupt() {
    // simulate interrupts
    if rand::random::<u32>() % 3 == 0 {
        thread::yield_now();
    }
}

fn inc(delta: u32, pair: Arc<(Mutex<u32>, Condvar)>) {
    for _ in 0..NUM_ITERS {
        let (lock, cvar) = &*pair;
        let mut val = lock.lock();
        *val += delta;
        may_interrupt();
        drop(val);
        may_interrupt();
        // We notify the condvar that the value has changed.
        cvar.notify_one();
    }
}

#[test]
fn test_wait() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let pair = Arc::new((Mutex::new(INIT_VALUE), Condvar::new()));
    for _ in 0..NUM_TASKS {
        let pair1 = Arc::clone(&pair);
        thread::spawn(move || inc(1, pair1));
        let pair2 = Arc::clone(&pair);
        thread::spawn(move || inc(2, pair2));
    }

    // Wait for the thread to start up.
    let (lock, cvar) = &*pair;
    let mut val = lock.lock();
    // As long as the value inside the `Mutex<usize>` is not `i`, we wait.
    while *val != NUM_ITERS * NUM_TASKS * 3 {
        may_interrupt();
        val = cvar.wait(val);
        may_interrupt();
    }
    drop(val);

    assert!(lock.lock().eq(&(NUM_ITERS * NUM_TASKS * 3)));

    println!("Condvar wait test OK");
}

#[test]
fn test_wait_while() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let pair = Arc::new((Mutex::new(INIT_VALUE), Condvar::new()));
    for _ in 0..NUM_TASKS {
        let pair1 = Arc::clone(&pair);
        thread::spawn(move || inc(1, pair1));
        let pair2 = Arc::clone(&pair);
        thread::spawn(move || inc(2, pair2));
    }

    // Wait for the thread to start up.
    let (lock, cvar) = &*pair;
    // As long as the value inside the `Mutex<bool>` is `true`, we wait.
    let val = cvar.wait_while(lock.lock(), |val| *val != NUM_ITERS * NUM_TASKS * 3);

    assert!(val.eq(&(NUM_ITERS * NUM_TASKS * 3)));

    println!("Condvar wait_while test OK");
}

#[test]
fn smoke() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let c = Condvar::new();
    c.notify_one();
    c.notify_all();
}

#[test]
fn notify_one() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let m = Arc::new(Mutex::new(()));
    let m2 = m.clone();
    let c = Arc::new(Condvar::new());
    let c2 = c.clone();

    let g = m.lock();
    let _t = thread::spawn(move || {
        let _g = m2.lock();
        c2.notify_one();
    });
    let g = c.wait(g);
    drop(g);
}

#[test]
fn notify_all() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let data = Arc::new((Mutex::new(0), Condvar::new()));
    let (tx, rx) = channel();
    for _ in 0..NUM_TASKS {
        let data = data.clone();
        let tx = tx.clone();
        thread::spawn(move || {
            let &(ref lock, ref cond) = &*data;
            let mut cnt = lock.lock();
            *cnt += 1;
            if *cnt == NUM_TASKS {
                tx.send(()).unwrap();
            }
            while *cnt != 0 {
                cnt = cond.wait(cnt);
            }
            tx.send(()).unwrap();
        });
    }
    drop(tx);

    let &(ref lock, ref cond) = &*data;
    // Yield manually to get tx.send() executed.
    thread::yield_now();
    rx.recv().unwrap();

    let mut cnt = lock.lock();
    *cnt = 0;
    cond.notify_all();
    drop(cnt);

    for _ in 0..NUM_TASKS {
        // Yield manually to get tx.send() executed.
        thread::yield_now();
        rx.recv().unwrap();
    }
}

#[test]
fn wait_while() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = pair.clone();

    // Inside of our lock, spawn a new thread, and then wait for it to start.
    thread::spawn(move || {
        let &(ref lock, ref cvar) = &*pair2;
        let mut started = lock.lock();
        *started = true;
        // We notify the condvar that the value has changed.
        cvar.notify_one();
    });

    // Wait for the thread to start up.
    let &(ref lock, ref cvar) = &*pair;
    let guard = cvar.wait_while(lock.lock(), |started| !*started);
    assert!(*guard);
}
