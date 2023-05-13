pub mod spin_mutex;
pub mod rw_spin_mutex;

/// Low-level support for mutex
pub trait MutexSupport {
    type GuardData;
    /// Called before lock() & try_lock()
    fn before_lock() -> Self::GuardData;
    /// Called when MutexGuard dropping
    fn after_unlock(_: &mut Self::GuardData);
}

/// 什么也不做的Spin
///
/// 谨防自旋锁中断死锁!
#[derive(Debug)]
pub struct Spin;

impl MutexSupport for Spin {
    type GuardData = ();
    #[inline(always)]
    fn before_lock() -> Self::GuardData {}
    #[inline(always)]
    fn after_unlock(_: &mut Self::GuardData) {}
}

#[allow(dead_code)]
#[inline(always)]
pub fn seq_fence() {
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}
