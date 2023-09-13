//! Define and access per-CPU data structures.
//!
//! All per-CPU data is placed into several contiguous memory regions called
//! **per-CPU data areas**, the number of which is the number of CPUs. Each CPU
//! has its own per-CPU data area. The architecture-specific thread pointer
//! register (e.g., `GS_BASE` on x86_64) is set to the base address of the area
//! on initialization.
//!
//! When accessing the per-CPU data on the current CPU, it first use the thread
//! pointer register to obtain the corresponding per-CPU data area, and then add
//! an offset to access the corresponding field.
//!
//! # Notes
//!
//! Since RISC-V does not provide separate thread pointer registers for user and
//! kernel mode, we temporarily use the `gp` register to point to the per-CPU data
//! area, while the `tp` register is used for thread-local storage.
//!
//! # Examples
//!
//! ```no_run
//! #[percpu::def_percpu]
//! static CPU_ID: usize = 0;
//!
//! // initialize per-CPU data for 4 CPUs.
//! percpu::init(4);
//! // set the thread pointer register to the per-CPU data area 0.
//! percpu::set_local_thread_pointer(0);
//!
//! // access the per-CPU data `CPU_ID` on the current CPU.
//! println!("{}", CPU_ID.read_current()); // prints "0"
//! CPU_ID.write_current(1);
//! println!("{}", CPU_ID.read_current()); // prints "1"
//! ```
//!
//! # Cargo Features
//!
//! - `sp-naive`: For **single-core** use. In this case, each per-CPU data is
//!    just a global variable, architecture-specific thread pointer register is
//!    not used.
//! - `preempt`: For **preemptible** system use. In this case, we need to disable
//!    preemption when accessing per-CPU data. Otherwise, the data may be corrupted
//!    when it's being accessing and the current thread happens to be preempted.

#![cfg_attr(target_os = "none", no_std)]
#![feature(doc_cfg)]

extern crate percpu_macros;

#[cfg_attr(feature = "sp-naive", path = "naive.rs")]
mod imp;

pub use self::imp::*;
pub use percpu_macros::def_percpu;

#[doc(hidden)]
pub mod __priv {
    #[cfg(feature = "preempt")]
    pub use kernel_guard::NoPreempt as NoPreemptGuard;
}

cfg_if::cfg_if! {
    if #[cfg(doc)] {
        /// Example per-CPU data for documentation only.
        #[doc(cfg(doc))]
        #[def_percpu]
        pub static EXAMPLE_PERCPU_DATA: usize = 0;
    }
}
