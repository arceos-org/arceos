use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;

use axtask as thread;

use crate::{Condvar, Mutex};

#[derive(Eq, PartialEq, Debug)]
struct NonCopy(i32);

struct Packet<T>(Arc<(Mutex<T>, Condvar)>);

fn may_interrupt() {
    // simulate interrupts
    if rand::random::<u32>() % 3 == 0 {
        thread::yield_now();
    }
}

#[test]
fn smoke() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let m = Mutex::new(());
    drop(m.lock());
    drop(m.lock());
}

#[test]
fn lots_and_lots() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

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

#[test]
fn try_lock() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let m = Mutex::new(());
    *m.try_lock().unwrap() = ();
}

#[test]
fn test_into_inner() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let m = Mutex::new(NonCopy(10));
    assert_eq!(m.into_inner(), NonCopy(10));
}

#[test]
fn test_into_inner_drop() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    struct Foo(Arc<AtomicUsize>);
    impl Drop for Foo {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let num_drops = Arc::new(AtomicUsize::new(0));
    let m = Mutex::new(Foo(num_drops.clone()));
    assert_eq!(num_drops.load(Ordering::SeqCst), 0);
    {
        let _inner = m.into_inner();
        assert_eq!(num_drops.load(Ordering::SeqCst), 0);
    }
    assert_eq!(num_drops.load(Ordering::SeqCst), 1);
}

#[test]
fn test_get_mut() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let mut m = Mutex::new(NonCopy(10));
    *m.get_mut() = NonCopy(20);
    assert_eq!(m.into_inner(), NonCopy(20));
}

#[test]
fn test_mutex_arc_condvar() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let packet = Packet(Arc::new((Mutex::new(false), Condvar::new())));
    let packet2 = Packet(packet.0.clone());
    let (tx, rx) = channel();
    let _t = thread::spawn(move || {
        // wait until parent gets in
        rx.recv().unwrap();
        let &(ref lock, ref cvar) = &*packet2.0;
        let mut lock = lock.lock();
        *lock = true;
        cvar.notify_one();
    });

    let &(ref lock, ref cvar) = &*packet.0;
    let mut lock = lock.lock();
    tx.send(()).unwrap();
    assert!(!*lock);
    while !*lock {
        lock = cvar.wait(lock);
    }
}

#[test]
fn test_mutex_arc_nested() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    // Tests nested mutexes and access
    // to underlying data.
    let arc = Arc::new(Mutex::new(1));
    let arc2 = Arc::new(Mutex::new(arc));
    let (tx, rx) = channel();
    let _t = thread::spawn(move || {
        let lock = arc2.lock();
        let lock2 = lock.lock();
        assert_eq!(*lock2, 1);
        tx.send(()).unwrap();
    });
    // Yield manually to get tx.send() executed.
    thread::yield_now();
    rx.recv().unwrap();
}

#[test]
fn test_mutex_unsized() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let mutex: &Mutex<[i32]> = &Mutex::new([1, 2, 3]);
    {
        let b = &mut *mutex.lock();
        b[0] = 4;
        b[2] = 5;
    }
    let comp: &[i32] = &[4, 2, 5];
    assert_eq!(&*mutex.lock(), comp);
}
