use alloc::collections::btree_map::BTreeMap;
use core::{
    alloc::Layout,
    ops::Range,
    sync::atomic::{AtomicBool, Ordering},
};

use axbacktrace::Backtrace;
use kspin::SpinNoIrq;

pub(crate) static TRACKING_ENABLED: AtomicBool = AtomicBool::new(false);

#[percpu::def_percpu]
pub(crate) static IN_GLOBAL_ALLOCATOR: bool = false;

/// Metadata for each allocation made by the global allocator.
#[derive(Debug)]
pub struct AllocationInfo {
    /// Layout of the allocation.
    pub layout: Layout,
    /// Backtrace at the time of allocation.
    pub backtrace: Backtrace,
    /// Generation at which the allocation was made.
    pub generation: u64,
}

pub(crate) struct GlobalState {
    // FIXME: don't know why using HashMap causes crash
    pub map: BTreeMap<usize, AllocationInfo>,
    pub generation: u64,
}

static STATE: SpinNoIrq<GlobalState> = SpinNoIrq::new(GlobalState {
    map: BTreeMap::new(),
    generation: 0,
});

/// Enables allocation tracking.
pub fn enable_tracking() {
    TRACKING_ENABLED.store(true, Ordering::SeqCst);
}

/// Disables allocation tracking.
pub fn disable_tracking() {
    TRACKING_ENABLED.store(false, Ordering::SeqCst);
}

/// Returns whether allocation tracking is enabled.
pub fn tracking_enabled() -> bool {
    TRACKING_ENABLED.load(Ordering::SeqCst)
}

pub(crate) fn with_state<R>(f: impl FnOnce(Option<&mut GlobalState>) -> R) -> R {
    IN_GLOBAL_ALLOCATOR.with_current(|in_global| {
        if *in_global || !tracking_enabled() {
            f(None)
        } else {
            *in_global = true;
            let mut state = STATE.lock();
            let result = f(Some(&mut state));
            *in_global = false;
            result
        }
    })
}

/// Returns the current generation of the global allocator.
///
/// The generation is incremented every time a new allocation is made. It
/// can be utilized to track the changes in the allocation state over time.
///
/// See [`allocations_in`].
pub fn current_generation() -> u64 {
    STATE.lock().generation
}

/// Visits all allocations made by the global allocator within the given
/// generation range.
pub fn allocations_in(range: Range<u64>, visitor: impl FnMut(&AllocationInfo)) {
    with_state(|state| {
        state
            .unwrap()
            .map
            .values()
            .filter(move |info| range.contains(&info.generation))
            .for_each(visitor)
    });
}
