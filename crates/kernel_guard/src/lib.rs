//! RAII wrappers to create a critical section with local IRQs or preemption
//! disabled, used to implement spin locks in kernel.
//!
//! The critical section is created after the guard struct is created, and is
//! ended when the guard falls out of scope.
//!
//! The crate user must implement the [`KernelGuardIf`] trait using
//! [`crate_interface::impl_interface`] to provide the low-level implementantion
//! of how to enable/disable kernel preemption, if the feature `preempt` is
//! enabled.
//!
//! Available guards:
//!
//! - [`NoOp`]: Does nothing around the critical section.
//! - [`IrqSave`]: Disables/enables local IRQs around the critical section.
//! - [`NoPreempt`]: Disables/enables kernel preemption around the critical
//!   section.
//! - [`NoPreemptIrqSave`]: Disables/enables both kernel preemption and local
//!   IRQs around the critical section.
//!
//! # Crate features
//!
//! - `preempt`: Use in the preemptive system. If this feature is enabled, you
//!    need to implement the [`KernelGuardIf`] trait in other crates. Otherwise
//!    the preemption enable/disable operations will be no-ops. This feature is
//!    disabled by default.
//!
//! # Examples
//!
//! ```
//! use kernel_guard::{KernelGuardIf, NoPreempt};
//!
//! struct KernelGuardIfImpl;
//!
//! #[crate_interface::impl_interface]
//! impl KernelGuardIf for KernelGuardIfImpl {
//!     fn enable_preempt() {
//!         // Your implementation here
//!     }
//!     fn disable_preempt() {
//!         // Your implementation here
//!     }
//! }
//!
//! let guard = NoPreempt::new();
//! /* The critical section starts here
//!
//! Do something that requires preemption to be disabled
//!
//! The critical section ends here */
//! drop(guard);
//! ```

#![no_std]
#![feature(asm_const)]

mod arch;

/// Low-level interfaces that must be implemented by the crate user.
#[crate_interface::def_interface]
pub trait KernelGuardIf {
    /// How to enable kernel preemption.
    fn enable_preempt();

    /// How to disable kernel preemption.
    fn disable_preempt();
}

/// A base trait that all guards implement.
pub trait BaseGuard {
    /// The saved state when entering the critical section.
    type State: Clone + Copy;

    /// Something that must be done before entering the critical section.
    fn acquire() -> Self::State;

    /// Something that must be done after leaving the critical section.
    fn release(state: Self::State);
}

/// A no-op guard that does nothing around the critical section.
pub struct NoOp;

cfg_if::cfg_if! {
    // For user-mode std apps, we use the alias of [`NoOp`] for all guards,
    // since we can not disable IRQs or preemption in user-mode.
    if #[cfg(any(target_os = "none", doc))] {
        /// A guard that disables/enables local IRQs around the critical section.
        pub struct IrqSave(usize);

        /// A guard that disables/enables kernel preemption around the critical
        /// section.
        pub struct NoPreempt;

        /// A guard that disables/enables both kernel preemption and local IRQs
        /// around the critical section.
        ///
        /// When entering the critical section, it disables kernel preemption
        /// first, followed by local IRQs. When leaving the critical section, it
        /// re-enables local IRQs first, followed by kernel preemption.
        pub struct NoPreemptIrqSave(usize);
    } else {
        /// Alias of [`NoOp`].
        pub type IrqSave = NoOp;

        /// Alias of [`NoOp`].
        pub type NoPreempt = NoOp;

        /// Alias of [`NoOp`].
        pub type NoPreemptIrqSave = NoOp;
    }
}

impl BaseGuard for NoOp {
    type State = ();
    fn acquire() -> Self::State {}
    fn release(_state: Self::State) {}
}

impl NoOp {
    /// Creates a new [`NoOp`] guard.
    pub const fn new() -> Self {
        Self
    }
}

impl Drop for NoOp {
    fn drop(&mut self) {}
}

#[cfg(any(target_os = "none", doc))]
mod imp {
    use super::*;

    impl BaseGuard for IrqSave {
        type State = usize;

        #[inline]
        fn acquire() -> Self::State {
            super::arch::local_irq_save_and_disable()
        }

        #[inline]
        fn release(state: Self::State) {
            // restore IRQ states
            super::arch::local_irq_restore(state);
        }
    }

    impl BaseGuard for NoPreempt {
        type State = ();
        fn acquire() -> Self::State {
            // disable preempt
            #[cfg(feature = "preempt")]
            crate_interface::call_interface!(KernelGuardIf::disable_preempt);
        }
        fn release(_state: Self::State) {
            // enable preempt
            #[cfg(feature = "preempt")]
            crate_interface::call_interface!(KernelGuardIf::enable_preempt);
        }
    }

    impl BaseGuard for NoPreemptIrqSave {
        type State = usize;
        fn acquire() -> Self::State {
            // disable preempt
            #[cfg(feature = "preempt")]
            crate_interface::call_interface!(KernelGuardIf::disable_preempt);
            // disable IRQs and save IRQ states
            super::arch::local_irq_save_and_disable()
        }
        fn release(state: Self::State) {
            // restore IRQ states
            super::arch::local_irq_restore(state);
            // enable preempt
            #[cfg(feature = "preempt")]
            crate_interface::call_interface!(KernelGuardIf::enable_preempt);
        }
    }

    impl IrqSave {
        /// Creates a new [`IrqSave`] guard.
        pub fn new() -> Self {
            Self(Self::acquire())
        }
    }

    impl Drop for IrqSave {
        fn drop(&mut self) {
            Self::release(self.0)
        }
    }

    impl Default for IrqSave {
        fn default() -> Self {
            Self::new()
        }
    }

    impl NoPreempt {
        /// Creates a new [`NoPreempt`] guard.
        pub fn new() -> Self {
            Self::acquire();
            Self
        }
    }

    impl Drop for NoPreempt {
        fn drop(&mut self) {
            Self::release(())
        }
    }

    impl Default for NoPreempt {
        fn default() -> Self {
            Self::new()
        }
    }

    impl NoPreemptIrqSave {
        /// Creates a new [`NoPreemptIrqSave`] guard.
        pub fn new() -> Self {
            Self(Self::acquire())
        }
    }

    impl Drop for NoPreemptIrqSave {
        fn drop(&mut self) {
            Self::release(self.0)
        }
    }

    impl Default for NoPreemptIrqSave {
        fn default() -> Self {
            Self::new()
        }
    }
}
