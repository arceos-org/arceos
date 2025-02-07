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

    // all threads share the same namespace
    assert_eq!(FOO.load(Ordering::SeqCst), 103);
    assert_eq!(BAR.lock().unwrap().as_str(), "one hello hello hello");
}

#[test]
fn test_namespace() {
    thread_init_namespace();
    FOO.init_new(100.into());
    BAR.init_new(Mutex::new(String::from("one")));

    let t1 = thread::spawn(|| {
        thread_init_namespace();
        thread_fn();
    });
    let t2 = thread::spawn(|| {
        thread_init_namespace();
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
            ns.init_once(AxNamespace::global());
        });
    }
}
