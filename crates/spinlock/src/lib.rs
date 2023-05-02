//! `no_std` spin lock implementation that can disable kernel local IRQs or
//! preemption while locking.
//!
//! # Cargo Features
//!
//! - `smp`: Use in the **multi-core** environment. For **single-core**
//!   environment (without this feature), the lock state is unnecessary and
//!   optimized out. CPU can always get the lock if we follow the proper guard
//!   in use. By default, this feature is disabled.

#![cfg_attr(not(test), no_std)]

mod base;

use kernel_guard::{NoOp, NoPreempt, NoPreemptIrqSave};

pub use self::base::{BaseSpinLock, BaseSpinLockGuard};

/// A spin lock that disables kernel preemption while trying to lock, and
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

/// A guard that provides mutable data access for [`SpinRaw`].
pub type SpinRawGuard<'a, T> = BaseSpinLockGuard<'a, NoOp, T>;
