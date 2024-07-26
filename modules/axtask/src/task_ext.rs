//! User-defined task extended data.

use core::mem::{align_of, size_of};

#[no_mangle]
#[linkage = "weak"]
static __AX_TASK_EXT_SIZE: usize = 0;

#[no_mangle]
#[linkage = "weak"]
static __AX_TASK_EXT_ALIGN: usize = 0;

pub(crate) struct AxTaskExt {
    ptr: *mut u8,
}

impl AxTaskExt {
    pub fn size() -> usize {
        extern "C" {
            static __AX_TASK_EXT_SIZE: usize;
        }
        unsafe { __AX_TASK_EXT_SIZE }
    }

    pub fn align() -> usize {
        extern "C" {
            static __AX_TASK_EXT_ALIGN: usize;
        }
        unsafe { __AX_TASK_EXT_ALIGN }
    }

    pub const fn null() -> Self {
        Self {
            ptr: core::ptr::null_mut(),
        }
    }

    pub unsafe fn uninited() -> Self {
        let size = Self::size();
        let align = Self::align();
        let ptr = if size == 0 {
            core::ptr::null_mut()
        } else {
            let layout = core::alloc::Layout::from_size_align(size, align).unwrap();
            unsafe { alloc::alloc::alloc(layout) }
        };
        Self { ptr }
    }

    pub const fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn init<T: Sized>(&mut self, data: T) {
        let data_size = size_of::<T>();
        let data_align = align_of::<T>();
        if data_size != Self::size() {
            panic!("size mismatch: {} != {}", data_size, Self::size());
        }
        if data_align != Self::align() {
            panic!("align mismatch: {} != {}", data_align, Self::align());
        }

        if self.ptr.is_null() {
            *self = unsafe { Self::uninited() };
        }
        if data_size > 0 {
            unsafe { (self.ptr as *mut T).write(data) };
        }
    }
}

impl Drop for AxTaskExt {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            let layout = core::alloc::Layout::from_size_align(Self::size(), 0x10).unwrap();
            unsafe { alloc::alloc::dealloc(self.ptr, layout) };
        }
    }
}

/// A trait to convert [`TaskInner::task_ext_ptr`] to the reference of the
/// concrete type.
///
/// [`TaskInner::task_ext_ptr`]: crate::TaskInner::task_ext_ptr
pub trait TaskExtRef<T: Sized> {
    /// Get a reference to the task extended data.
    fn task_ext(&self) -> &T;
}

/// A trait to convert [`TaskInner::task_ext_ptr`] to the mutable reference of
/// the concrete type.
///
/// [`TaskInner::task_ext_ptr`]: crate::TaskInner::task_ext_ptr
pub trait TaskExtMut<T: Sized> {
    /// Get a mutable reference to the task extended data.
    fn task_ext_mut(&mut self) -> &mut T;
}

/// Define the task extended data.
///
/// It automatically implements [`TaskExtRef`] and [`TaskExtMut`] for
/// [`TaskInner`].
///
/// # Example
///
/// ```
/// # #![allow(non_local_definitions)]
/// use axtask::{def_task_ext, TaskExtRef, TaskInner};
///
/// pub struct TaskExtImpl {
///    proc_id: usize,
/// }
///
/// def_task_ext!(TaskExtImpl);
///
/// axtask::init_scheduler();
///
/// let mut inner = TaskInner::new(|| {},  "".into(), 0x1000);
/// inner.init_task_ext(TaskExtImpl { proc_id: 233 });
/// let task = axtask::spawn_task(inner);
/// assert_eq!(task.task_ext().proc_id, 233);
/// ```
///
/// [`TaskInner`]: crate::TaskInner
#[macro_export]
macro_rules! def_task_ext {
    ($task_ext_struct:ty) => {
        #[no_mangle]
        static __AX_TASK_EXT_SIZE: usize = ::core::mem::size_of::<$task_ext_struct>();

        #[no_mangle]
        static __AX_TASK_EXT_ALIGN: usize = ::core::mem::align_of::<$task_ext_struct>();

        impl $crate::TaskExtRef<$task_ext_struct> for $crate::TaskInner {
            fn task_ext(&self) -> &$task_ext_struct {
                unsafe { &*(self.task_ext_ptr() as *const $task_ext_struct) }
            }
        }

        impl $crate::TaskExtMut<$task_ext_struct> for $crate::TaskInner {
            fn task_ext_mut(&mut self) -> &mut $task_ext_struct {
                unsafe { &mut *(self.task_ext_ptr() as *mut $task_ext_struct) }
            }
        }
    };
}
