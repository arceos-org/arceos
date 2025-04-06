//! [ArceOS](https://github.com/arceos-org/arceos) namespaces module.
//!
//! Namespaces are used to control system resource sharing between threads. This
//! module provides a unified interface to access system resources in different
//! scenarios.
//!
//! For a unikernel, there is only one global namespace, so all threads share
//! the same system resources, such as virtual address space, working directory,
//! and file descriptors, etc.
//!
//! For a monolithic kernel, each process corresponds to a namespace, all
//! threads in the same process share the same system resources. Different
//! processes have different namespaces and isolated resources.
//!
//! For further container support, some global system resources can also be
//! grouped into a namespace.
//!
//! See the examples of [`def_resource!`] for more usage.

#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::sync::Arc;
use core::{alloc::Layout, fmt, ops::Deref};

use lazyinit::LazyInit;

unsafe extern "C" {
    fn __start_axns_resource();
    fn __stop_axns_resource();
}

/// A namespace that contains all user-defined resources.
///
/// There are two types of namespaces:
///
/// - Global namespace: this namespace is globally unique and all threads share
///   the resources in it. Resources are statically collected into the
///   `axns_resource` section, and the global namespace is constructed by the base
///   address of the section ([`AxNamespace::global`]).
/// - Thread-local namespace: this namespace is per-thread, each thread should
///   call [`AxNamespace::new_thread_local()`] to allocate a memory area as its
///   namespace. Layout of resources in global and thread-local namespaces is
///   consistent. Each namespace has its own resources, which may be unique or
///   shared between threads by the [`Arc`] wrapper.
pub struct AxNamespace {
    base: usize,
    alloc: bool,
}

impl AxNamespace {
    /// Returns the base address of the namespace, which points to the start of
    /// all resources.
    pub const fn base(&self) -> *mut u8 {
        self.base as *mut u8
    }

    /// Returns the size of the namespace (size of all resources).
    pub fn size(&self) -> usize {
        Self::section_size()
    }

    /// Returns the size of the `axns_resource` section.
    fn section_size() -> usize {
        __stop_axns_resource as usize - __start_axns_resource as usize
    }

    /// Returns the global namespace.
    pub fn global() -> Self {
        Self {
            base: __start_axns_resource as usize,
            alloc: false,
        }
    }

    /// Constructs a new thread-local namespace.
    ///
    /// Each thread can have its own namespace instead of the global one, to
    /// isolate resources between threads.
    ///
    /// This function allocates a memory area to store the thread-local resources,
    /// and copies from the global namespace as the initial value.
    #[cfg(feature = "thread-local")]
    pub fn new_thread_local() -> Self {
        let size = Self::section_size();
        let base = if size == 0 {
            core::ptr::null_mut()
        } else {
            let layout = Layout::from_size_align(size, 64).unwrap();
            let dst = unsafe { alloc::alloc::alloc(layout) };
            let src = __start_axns_resource as *const u8;
            unsafe { core::ptr::copy_nonoverlapping(src, dst, size) };
            dst
        } as usize;
        Self { base, alloc: true }
    }
}

impl Drop for AxNamespace {
    fn drop(&mut self) {
        if self.alloc {
            let size = Self::section_size();
            let base = self.base();
            if size != 0 && !base.is_null() {
                let layout = Layout::from_size_align(size, 64).unwrap();
                unsafe { alloc::alloc::dealloc(base, layout) };
            }
        }
    }
}

/// A helper type to easily manage shared resources.
///
/// It provides methods to lazily initialize the resource of the current thread,
/// or to share the resource with other threads.
pub struct ResArc<T>(LazyInit<Arc<T>>);

impl<T> ResArc<T> {
    /// Creates a new uninitialized resource.
    pub const fn new() -> Self {
        Self(LazyInit::new())
    }

    /// Returns a shared reference to the resource.
    pub fn share(&self) -> Arc<T> {
        self.0.deref().clone()
    }

    /// Initializes the resource and does not share with others.
    pub fn init_new(&self, data: T) {
        self.0.init_once(Arc::new(data));
    }

    /// Initializes the resource with the shared data.
    pub fn init_shared(&self, data: Arc<T>) {
        self.0.init_once(data);
    }

    /// Checks whether the value is initialized.
    pub fn is_inited(&self) -> bool {
        self.0.is_inited()
    }
}

impl<T> Deref for ResArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: fmt::Debug> fmt::Debug for ResArc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The interfaces need to be implemented when enable thread-local namespaces.
#[cfg(feature = "thread-local")]
#[crate_interface::def_interface]
pub trait AxNamespaceIf {
    /// Returns the pointer to the current namespace.
    ///
    /// It usually needs to be obtained from the thread local storage.
    fn current_namespace_base() -> *mut u8;
}

/// Returns the pointer to the current namespace.
///
/// When `thread-local` feature is enabled, it returns the thread-local namespace
/// of the current thread. Otherwise, it returns the global namespace.
///
/// # Safety
///
/// This function is unsafe, the returned pointer should not outlive the current
/// thread.
pub unsafe fn current_namespace_base() -> *mut u8 {
    #[cfg(feature = "thread-local")]
    {
        crate_interface::call_interface!(AxNamespaceIf::current_namespace_base)
    }
    #[cfg(not(feature = "thread-local"))]
    {
        AxNamespace::global().base()
    }
}

/// Defines a resource that managed by [`AxNamespace`].
///
/// Each resource will be collected into the `axns_resource` section. When
/// accessed, it is either dereferenced from the global namespace or the
/// thread-local namespace according to the `thread-local` feature.
///
/// # Example
///
/// ```
/// use axns::ResArc;
///
/// axns::def_resource! {
///     static FOO: u32 = 42;
///     static BAR: ResArc<String> = ResArc::new();
/// }
///
/// BAR.init_new("hello world".to_string());
/// assert_eq!(*FOO, 42);
/// assert_eq!(BAR.as_str(), "hello world");
///
/// mod imp {
///     use axns::{AxNamespace, AxNamespaceIf};
///
///     struct ResArcImpl;
///
///     #[crate_interface::impl_interface]
///     impl AxNamespaceIf for ResArcImpl {
///         fn current_namespace_base() -> *mut u8 {
///             AxNamespace::global().base()
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! def_resource {
    ( $( $(#[$attr:meta])* $vis:vis static $name:ident: $ty:ty = $default:expr; )+ ) => {
        $(
            #[doc = concat!("Wrapper struct for the namespace resource [`", stringify!($name), "`]")]
            #[allow(non_camel_case_types)]
            $vis struct $name { __value: () }

            impl $name {
                unsafe fn deref_from_base(&self, ns_base: *mut u8) -> &$ty {
                    unsafe extern {
                        fn __start_axns_resource();
                    }

                    #[unsafe(link_section = "axns_resource")]
                    static RES: $ty = $default;

                    let offset = &RES as *const _ as usize - __start_axns_resource as usize;
                    let ptr = unsafe{ ns_base.add(offset) } as *const _;
                    unsafe{ &*ptr }
                }

                /// Dereference the resource from the given namespace.
                pub fn deref_from(&self, ns: &$crate::AxNamespace) -> &$ty {
                    unsafe { self.deref_from_base(ns.base()) }
                }

                /// Dereference the resource from the global namespace.
                pub fn deref_global(&self) -> &$ty {
                    self.deref_from(&$crate::AxNamespace::global())
                }

                /// Dereference the resource automatically, according whether the
                /// `thread-local` feature of the `axns` crate is enabled or not.
                ///
                /// When the feature is enabled, it dereferences from the
                /// thread-local namespace of the current thread. Otherwise, it
                /// dereferences from the global namespace.
                pub fn deref_auto(&self) -> &$ty {
                    unsafe { self.deref_from_base($crate::current_namespace_base()) }
                }
            }

            impl core::ops::Deref for $name {
                type Target = $ty;

                #[inline(never)]
                fn deref(&self) -> &Self::Target {
                    self.deref_auto()
                }
            }

            #[used]
            #[doc(hidden)]
            $(#[$attr])*
            $vis static $name: $name = $name { __value: () };
        )+
    };
}
