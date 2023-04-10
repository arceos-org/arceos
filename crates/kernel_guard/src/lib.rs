//! RAII wrappers to create a critical section with local IRQs or preemption
//! disabled, used to implement spin locks in kernel.

#![no_std]
#![feature(asm_const)]
#![allow(clippy::new_without_default)]

mod arch;

#[crate_interface::def_interface]
pub trait KernelGuardIf {
    fn enable_preempt();
    fn disable_preempt();
}

pub trait BaseGuard {
    type State: Clone + Copy;
    fn acquire() -> Self::State;
    fn release(state: Self::State);
}

pub struct NoOp;

cfg_if::cfg_if! {
    // For user-mode std apps, we use the alias of [`NoOp`] for all guards,
    // since we can not disable IRQs or preemption in user-mode.
    if #[cfg(target_os = "none")] {
        pub struct IrqSave(usize);
        pub struct NoPreempt;
        pub struct NoPreemptIrqSave(usize);
    } else {
        pub type IrqSave = NoOp;
        pub type NoPreempt = NoOp;
        pub type NoPreemptIrqSave = NoOp;
    }
}

impl BaseGuard for NoOp {
    type State = ();
    fn acquire() -> Self::State {}
    fn release(_state: Self::State) {}
}

impl NoOp {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "none")]
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
            crate_interface::call_interface!(KernelGuardIf::disable_preempt);
        }
        fn release(_state: Self::State) {
            // enable preempt
            crate_interface::call_interface!(KernelGuardIf::enable_preempt);
        }
    }

    impl BaseGuard for NoPreemptIrqSave {
        type State = usize;
        fn acquire() -> Self::State {
            // disable preempt
            crate_interface::call_interface!(KernelGuardIf::disable_preempt);
            // disable IRQs and save IRQ states
            super::arch::local_irq_save_and_disable()
        }
        fn release(state: Self::State) {
            // restore IRQ states
            super::arch::local_irq_restore(state);
            // enable preempt
            crate_interface::call_interface!(KernelGuardIf::enable_preempt);
        }
    }

    impl IrqSave {
        pub fn new() -> Self {
            Self(Self::acquire())
        }
    }

    impl Drop for IrqSave {
        fn drop(&mut self) {
            Self::release(self.0)
        }
    }

    impl NoPreempt {
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

    impl NoPreemptIrqSave {
        pub fn new() -> Self {
            Self(Self::acquire())
        }
    }

    impl Drop for NoPreemptIrqSave {
        fn drop(&mut self) {
            Self::release(self.0)
        }
    }
}
