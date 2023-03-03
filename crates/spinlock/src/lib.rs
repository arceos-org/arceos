#![cfg_attr(not(test), no_std)]
#![feature(asm_const)]
#![feature(const_trait_impl)]

mod arch;
mod base;

use self::base::{BaseSpinLock, BaseSpinLockGuard};
use crate_interface::def_interface;

/// A spin lock that disbales kernel preemption while trying to lock, and
/// re-enables it after unlocking.
///
/// It must be used in the local IRQ-disabled context, or never be used in
/// interrupt handlers.
pub type SpinNoPreempt<T> = BaseSpinLock<NoPreempt, T>;

/// A guard that provides mutable data access for [`SpinNoPreempt`].
pub type SpinNoPreemptGuard<'a, T> = BaseSpinLockGuard<'a, NoPreempt, T>;

/// A spin lock that disables kernel preemption and local IRQs while trying to
/// lock, and re-enables it after unlocking.
///
/// It can be used in the IRQ-enabled context.
pub type SpinNoIrq<T> = BaseSpinLock<NoPreemptIrqSave, T>;

/// A guard that provides mutable data access for [`SpinNoIrq`].
pub type SpinNoIrqGuard<'a, T> = BaseSpinLockGuard<'a, NoPreemptIrqSave, T>;

/// A raw spin lock that does nothing while trying to lock.
///
/// It must be used in the preemption-disabled and local IRQ-disabled context,
/// or never be used in interrupt handlers.
pub type SpinRaw<T> = BaseSpinLock<NoOp, T>;

/// A guard that provides mutable data access for [`SpinLockRaw`].
pub type SpinRawGuard<'a, T> = BaseSpinLockGuard<'a, NoOp, T>;

#[def_interface]
pub trait SpinLockIf {
    /// Enable or disable kernel preemption.
    fn set_preemptible(enabled: bool);
}

pub trait SpinLockStrategy {
    type Flags: Clone + Copy;
    fn acquire() -> Self::Flags;
    fn release(flags: Self::Flags);
}

pub struct NoOp;

cfg_if::cfg_if! {
    // For user-mode std apps, we use aliases for [`SpinLock`] and [`SpinNoIrq`],
    // since we can not disable IRQs or preemption in user-mode.
    if #[cfg(not(feature = "std"))] {
        pub struct NoPreempt;
        pub struct NoPreemptIrqSave(usize);
    } else {
        pub type NoPreempt = NoOp;
        pub type NoPreemptIrqSave = NoOp;
    }
}

impl SpinLockStrategy for NoOp {
    type Flags = ();
    fn acquire() -> Self::Flags {}
    fn release(_flags: Self::Flags) {}
}

#[cfg(not(feature = "std"))]
impl SpinLockStrategy for NoPreempt {
    type Flags = ();
    fn acquire() -> Self::Flags {
        // disable preempt
        #[cfg(not(feature = "std"))]
        crate_interface::call_interface!(SpinLockIf::set_preemptible, false);
    }
    fn release(_flags: Self::Flags) {
        // enable preempt
        #[cfg(not(feature = "std"))]
        crate_interface::call_interface!(SpinLockIf::set_preemptible, true);
    }
}

#[cfg(not(feature = "std"))]
impl SpinLockStrategy for NoPreemptIrqSave {
    type Flags = usize;
    fn acquire() -> Self::Flags {
        // disable preempt
        crate_interface::call_interface!(SpinLockIf::set_preemptible, false);
        // disable IRQs and save IRQ states
        crate::arch::local_irq_save_and_disable()
    }
    fn release(flags: Self::Flags) {
        // restore IRQ states
        crate::arch::local_irq_restore(flags);
        // enable preempt
        crate_interface::call_interface!(SpinLockIf::set_preemptible, true);
    }
}
