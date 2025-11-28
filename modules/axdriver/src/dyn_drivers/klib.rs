//! Platform implementation of the `axklib::Klib` trait.
//!
//! This crate provides the platform-side glue that implements the small set
//! of kernel helper functions defined in `axklib`. The implementation is
//! intentionally minimal: it forwards memory mapping requests to `axmm`,
//! delegates timing to `axhal`, and wires IRQ operations to `axhal` when the
//! `irq` feature is enabled.
//!
//! The implementation uses the `impl_trait!` helper to generate the FFI
//! shims expected by consumers. Documentation here focuses on the behavior
//! and expectations of each exported function.

use core::time::Duration;

use axklib::{AxResult, IrqHandler, Klib, PhysAddr, VirtAddr, impl_trait};

struct KlibImpl;

impl_trait! {
    impl Klib for KlibImpl {
        /// Map a physical region by delegating to the memory manager (`axmm`).
        ///
        /// This function forwards the request to `axmm::iomap` and returns the
        /// resulting virtual address wrapped in an `AxResult`.
        fn mem_iomap(addr: PhysAddr, size: usize) -> AxResult<VirtAddr> {
            axmm::iomap(addr, size)
        }

        /// Busy-wait for the given duration by calling into `axhal`.
        ///
        /// Short delays are serviced by the hardware abstraction layer's
        /// busy-wait implementation. This is suitable for small spin waits
        /// but should not be used for long sleeps.
        fn time_busy_wait(dur: Duration) {
            axhal::time::busy_wait(dur);
        }

        /// Enable or disable the specified IRQ line.
        ///
        /// When the `irq` feature is enabled this forwards to
        /// `axhal::irq::set_enable`. If the feature is not enabled the
        /// function currently panics via `unimplemented!()`; callers should
        /// avoid relying on IRQ operations when the platform omits IRQ
        /// support.
        fn irq_set_enable(_irq: usize, _enabled: bool) {
            #[cfg(feature = "irq")]
            axhal::irq::set_enable(_irq, _enabled);
            #[cfg(not(feature = "irq"))]
            unimplemented!();
        }

        /// Register an IRQ handler for the given IRQ number.
        ///
        /// Returns `true` when registration succeeds. With the `irq`
        /// feature enabled this delegates to `axhal::irq::register`.
        /// When IRQs are not enabled the function is currently unimplemented
        /// and will panic if called.
        fn irq_register(_irq: usize, _handler: IrqHandler) -> bool {
            #[cfg(feature = "irq")]
            {
                axhal::irq::register(_irq, _handler)
            }
            #[cfg(not(feature = "irq"))]
            {
                unimplemented!()
            }
        }
    }
}
