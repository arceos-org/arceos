//! [ArceOS] user program library, with an interface similar to rust
//! [std], but calling the functions directly
//! in ArceOS modules, instead of using libc and system calls.
//!
//! # Cargo Features
//!
//! - CPU
//!     - `smp`: Enable SMP (symmetric multiprocessing) support.
//!     - `fp_simd`: Enable floating point and SIMD support.
//! - Memory
//!     - `alloc`: Enable dynamic memory allocation.
//!     - `paging`: Enable page table manipulation.
//! - Task management
//!     - `multitask`: Enable multi-threading support.
//!     - `sched_fifo`: Use the FIFO cooperative scheduler.
//!     - `sched_rr`: Use the Round-robin preemptive scheduler.
//! - Device and upperlayer stack
//!     - `fs`: Enable file system support.
//!     - `net`: Enable networking support.
//!     - `display`: Enable graphics support.
//! - Logging
//!     - `log-level-off`: Disable all logging.
//!     - `log-level-error`, `log-level-warn`, `log-level-info`, `log-level-debug`,
//!       `log-level-trace`: Keep logging only at the specified level or higher.
//! - Platform
//!     - `platform-qemu-virt-riscv`: Specify for use on the corresponding platform.
//!     - `platform-qemu-virt-aarch64`: Specify for use on the corresponding platform.
//! - Other
//!    - `cbindings`: Generate C bindings, to support C applications.
//!
//! [ArceOS]: https://github.com/rcore-os/arceos

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_auto_cfg)]

pub use axlog::{debug, error, info, trace, warn};

#[cfg(not(test))]
extern crate axruntime;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
#[doc(no_inline)]
pub use alloc::{boxed, format, string, vec};

pub mod env;
pub mod io;
pub mod rand;
pub mod sync;
pub mod task;
pub mod time;

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "net")]
pub mod net;

#[cfg(feature = "display")]
pub mod display;

#[cfg(feature = "cbindings")]
pub mod cbindings;
