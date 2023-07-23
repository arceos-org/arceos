use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use core::cell::UnsafeCell;
use core::ffi::{c_int, c_void};

use axerrno::{LinuxError, LinuxResult};
use axtask::AxTaskRef;
use spin::RwLock;

use super::ctypes;

pub mod mutex;

lazy_static::lazy_static! {
    static ref TID_TO_PTHREAD: RwLock<BTreeMap<u64, ForceSendSync<ctypes::pthread_t>>> = {
        let mut map = BTreeMap::new();
        let main_task = axtask::current();
        let main_tid = main_task.id().as_u64();
        let main_thread = Pthread {
            inner: main_task.as_task_ref().clone(),
            retval: Arc::new(Packet {
                result: UnsafeCell::new(core::ptr::null_mut()),
            }),
        };
        let ptr = Box::into_raw(Box::new(main_thread)) as *mut c_void;
        map.insert(main_tid, ForceSendSync(ptr));
        RwLock::new(map)
    };
}

struct Packet<T> {
    result: UnsafeCell<T>,
}

unsafe impl<T> Send for Packet<T> {}
unsafe impl<T> Sync for Packet<T> {}

pub struct Pthread {
    inner: AxTaskRef,
    retval: Arc<Packet<*mut c_void>>,
}

impl Pthread {
    fn create(
        _attr: *const ctypes::pthread_attr_t,
        start_routine: extern "C" fn(arg: *mut c_void) -> *mut c_void,
        arg: *mut c_void,
    ) -> LinuxResult<ctypes::pthread_t> {
        let arg_wrapper = ForceSendSync(arg);

        let my_packet: Arc<Packet<*mut c_void>> = Arc::new(Packet {
            result: UnsafeCell::new(core::ptr::null_mut()),
        });
        let their_packet = my_packet.clone();

        let main = move || {
            let arg = arg_wrapper;
            let ret = start_routine(arg.0);
            unsafe { *their_packet.result.get() = ret };
            drop(their_packet);
        };

        let task_inner = axtask::spawn(main);
        let tid = task_inner.id().as_u64();
        let thread = Pthread {
            inner: task_inner,
            retval: my_packet,
        };
        let ptr = Box::into_raw(Box::new(thread)) as *mut c_void;
        TID_TO_PTHREAD.write().insert(tid, ForceSendSync(ptr));
        Ok(ptr)
    }

    fn current_ptr() -> *mut Pthread {
        let tid = axtask::current().id().as_u64();
        match TID_TO_PTHREAD.read().get(&tid) {
            None => core::ptr::null_mut(),
            Some(ptr) => ptr.0 as *mut Pthread,
        }
    }

    fn current() -> Option<&'static Pthread> {
        unsafe { core::ptr::NonNull::new(Self::current_ptr()).map(|ptr| ptr.as_ref()) }
    }

    fn exit_current(retval: *mut c_void) -> ! {
        let thread = Self::current().expect("fail to get current thread");
        unsafe { *thread.retval.result.get() = retval };
        axtask::exit(0);
    }

    fn join(ptr: ctypes::pthread_t) -> LinuxResult<*mut c_void> {
        if core::ptr::eq(ptr, Self::current_ptr() as _) {
            return Err(LinuxError::EDEADLK);
        }

        let thread = unsafe { Box::from_raw(ptr as *mut Pthread) };
        thread.inner.join();
        let tid = thread.inner.id().as_u64();
        let retval = unsafe { *thread.retval.result.get() };
        TID_TO_PTHREAD.write().remove(&tid);
        drop(thread);
        Ok(retval)
    }
}

/// Get current thread ID.
#[no_mangle]
pub unsafe extern "C" fn ax_getpid() -> c_int {
    ax_call_body!(ax_getpid, {
        let pid = axtask::current().id().as_u64() as c_int;
        Ok(pid)
    })
}

/// Returns the `pthread` struct of current thread.
#[no_mangle]
pub unsafe extern "C" fn ax_pthread_self() -> ctypes::pthread_t {
    Pthread::current().expect("fail to get current thread") as *const Pthread as _
}

/// Create a new thread with the given entry point and argument.
///
/// If successful, it stores the pointer to the newly created `struct __pthread`
/// in `res` and returns 0.
#[no_mangle]
pub unsafe extern "C" fn ax_pthread_create(
    res: *mut ctypes::pthread_t,
    attr: *const ctypes::pthread_attr_t,
    start_routine: extern "C" fn(arg: *mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    debug!(
        "ax_pthread_create <= {:#x}, {:#x}",
        start_routine as usize, arg as usize
    );
    ax_call_body!(ax_pthread_create, {
        let ptr = Pthread::create(attr, start_routine, arg)?;
        unsafe { core::ptr::write(res, ptr) };
        Ok(0)
    })
}

/// Exits the current thread. The value `retval` will be returned to the joiner.
#[no_mangle]
pub unsafe extern "C" fn ax_pthread_exit(retval: *mut c_void) -> ! {
    debug!("ax_pthread_exit <= {:#x}", retval as usize);
    Pthread::exit_current(retval);
}

/// Waits for the given thread to exit, and stores the return value in `retval`.
#[no_mangle]
pub unsafe extern "C" fn ax_pthread_join(
    thread: ctypes::pthread_t,
    retval: *mut *mut c_void,
) -> c_int {
    debug!("ax_pthread_join <= {:#x}", retval as usize);
    ax_call_body!(ax_pthread_join, {
        let ret = Pthread::join(thread)?;
        if !retval.is_null() {
            unsafe { core::ptr::write(retval, ret) };
        }
        Ok(0)
    })
}

#[derive(Clone, Copy)]
struct ForceSendSync<T>(T);

unsafe impl<T> Send for ForceSendSync<T> {}
unsafe impl<T> Sync for ForceSendSync<T> {}
