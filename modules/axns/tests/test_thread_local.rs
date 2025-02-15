#![feature(thread_id_value)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Barrier, Mutex};
use std::thread;

use axns::{ResArc, def_resource};

use self::imp::thread_init_namespace;

def_resource! {
    static FOO: ResArc<AtomicUsize> = ResArc::new();
    static BAR: ResArc<Mutex<String>> = ResArc::new();
}

static BARRIER: Barrier = Barrier::new(3);

fn thread_fn() {
    FOO.fetch_add(1, Ordering::SeqCst);
    BAR.lock().unwrap().push_str(" hello");

    BARRIER.wait();
    println!("{:?} FOO: {:?}", std::thread::current().id(), *FOO);
    println!("{:?} BAR: {:?}", std::thread::current().id(), BAR.lock());

    let id: u64 = thread::current().id().as_u64().into();
    if id == 2 || id == 4 {
        assert_eq!(FOO.load(Ordering::SeqCst), 102);
        assert_eq!(BAR.lock().unwrap().as_str(), "one hello hello");
    } else if id == 3 {
        assert_eq!(FOO.load(Ordering::SeqCst), 201);
        assert_eq!(BAR.lock().unwrap().as_str(), "two hello");
    }
}

#[test]
fn test_namespace() {
    thread_init_namespace();
    FOO.init_new(100.into());
    BAR.init_new(Mutex::new(String::from("one")));

    let t0_foo = FOO.share();
    let t0_bar = BAR.share();

    let t1 = thread::spawn(|| {
        thread_init_namespace();
        FOO.init_new(200.into()); // isolated from t0
        BAR.init_new(Mutex::new(String::from("two"))); // isolated from t0
        thread_fn();
    });
    let t2 = thread::spawn(|| {
        thread_init_namespace();
        FOO.init_shared(t0_foo); // shared with t0
        BAR.init_shared(t0_bar); // shared with t0
        thread_fn();
    });

    thread_fn();
    t1.join().unwrap();
    t2.join().unwrap();
}

mod imp {
    use axns::{AxNamespace, AxNamespaceIf};
    use lazyinit::LazyInit;

    thread_local! {
        static NS: LazyInit<AxNamespace> = LazyInit::new();
    }

    struct AxNamespaceImpl;

    #[crate_interface::impl_interface]
    impl AxNamespaceIf for AxNamespaceImpl {
        fn current_namespace_base() -> *mut u8 {
            NS.with(|ns| ns.base())
        }
    }

    pub fn thread_init_namespace() {
        NS.with(|ns| {
            ns.init_once(AxNamespace::new_thread_local());
        });
    }
}
