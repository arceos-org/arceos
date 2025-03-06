use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;

use axtask as thread;

use crate::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

const NUM_TASKS: u32 = 10;
const NUM_ITERS: u32 = 10_000;

#[derive(Eq, PartialEq, Debug)]
struct NonCopy(i32);

#[test]
fn smoke() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let l = RwLock::new(());
    drop(l.read());
    drop(l.write());
    drop((l.read(), l.read()));
    drop(l.write());
}

fn ramdom_bool() -> bool {
    rand::random::<u32>() % 2 == 0
}

#[test]
fn frob() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let r = Arc::new(RwLock::new(()));

    let (tx, rx) = channel::<()>();
    for _ in 0..NUM_TASKS {
        let tx = tx.clone();
        let r = r.clone();
        thread::spawn(move || {
            for _ in 0..NUM_ITERS {
                if ramdom_bool() {
                    drop(r.write());
                } else {
                    drop(r.read());
                }
            }
            drop(tx);
        });
    }
    drop(tx);
    // Yield manually to get tx.send() executed.
    thread::yield_now();
    let _ = rx.recv();
}

#[test]
fn test_rw_arc() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let arc = Arc::new(RwLock::new(0));
    let arc2 = arc.clone();
    let (tx, rx) = channel();

    thread::spawn(move || {
        let mut lock = arc2.write();
        for _ in 0..10 {
            let tmp = *lock;
            *lock = -1;
            thread::yield_now();
            *lock = tmp + 1;
        }
        tx.send(()).unwrap();
    });

    // Readers try to catch the writer in the act
    let mut children = Vec::new();
    for _ in 0..5 {
        let arc3 = arc.clone();
        children.push(thread::spawn(move || {
            let lock = arc3.read();
            assert!(*lock >= 0);
        }));
    }

    // Wait for children to pass their asserts
    for r in children {
        assert!(r.join().unwrap() == 0);
    }

    // Wait for writer to finish
    rx.recv().unwrap();
    let lock = arc.read();
    assert_eq!(*lock, 10);
}

#[test]
fn test_rwlock_unsized() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let rw: &RwLock<[i32]> = &RwLock::new([1, 2, 3]);
    {
        let b = &mut *rw.write();
        b[0] = 4;
        b[2] = 5;
    }
    let comp: &[i32] = &[4, 2, 5];
    assert_eq!(&*rw.read(), comp);
}

#[test]
fn test_rwlock_try_write() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let lock = RwLock::new(0isize);
    let read_guard = lock.read();

    let write_result = lock.try_write();
    match write_result {
        None => (),
        Some(_) => assert!(
            false,
            "try_write should not succeed while read_guard is in scope"
        ),
    }

    drop(read_guard);
    let mapped_read_guard = RwLockReadGuard::map(lock.read(), |_| &());

    let write_result = lock.try_write();
    match write_result {
        None => (),
        Some(_) => assert!(
            false,
            "try_write should not succeed while mapped_read_guard is in scope"
        ),
    }

    drop(mapped_read_guard);
}

#[test]
fn test_into_inner() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let m = RwLock::new(NonCopy(10));
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
    let m = RwLock::new(Foo(num_drops.clone()));
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

    let mut m = RwLock::new(NonCopy(10));
    *m.get_mut() = NonCopy(20);
    assert_eq!(m.into_inner(), NonCopy(20));
}

#[test]
fn test_read_guard_covariance() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    fn do_stuff<'a>(_: RwLockReadGuard<'_, &'a i32>, _: &'a i32) {}
    let j: i32 = 5;
    let lock = RwLock::new(&j);
    {
        let i = 6;
        do_stuff(lock.read(), &i);
    }
    drop(lock);
}

#[test]
fn test_mapped_read_guard_covariance() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    fn do_stuff<'a>(_: MappedRwLockReadGuard<'_, &'a i32>, _: &'a i32) {}
    let j: i32 = 5;
    let lock = RwLock::new((&j, &j));
    {
        let i = 6;
        let guard = lock.read();
        let guard = RwLockReadGuard::map(guard, |(val, _val)| val);
        do_stuff(guard, &i);
    }
    drop(lock);
}

#[test]
fn test_mapping_mapped_guard() {
    let _lock = crate::tests::SEQ.lock();
    crate::tests::INIT.call_once(thread::init_scheduler);

    let arr = [0; 4];
    let mut lock = RwLock::new(arr);
    let guard = lock.write();
    let guard = RwLockWriteGuard::map(guard, |arr| &mut arr[..2]);
    let mut guard = MappedRwLockWriteGuard::map(guard, |slice| &mut slice[1..]);
    assert_eq!(guard.len(), 1);
    guard[0] = 42;
    drop(guard);
    assert_eq!(*lock.get_mut(), [0, 42, 0, 0]);

    let guard = lock.read();
    let guard = RwLockReadGuard::map(guard, |arr| &arr[..2]);
    let guard = MappedRwLockReadGuard::map(guard, |slice| &slice[1..]);
    assert_eq!(*guard, [42]);
    drop(guard);
    assert_eq!(*lock.get_mut(), [0, 42, 0, 0]);
}
