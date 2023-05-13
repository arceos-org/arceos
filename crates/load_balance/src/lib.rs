//! Various scheduler algorithms in a unified interface.
//!
//! Currently supported algorithms:
//!
//! - [`FifoScheduler`]: FIFO (First-In-First-Out) scheduler (cooperative).
//! - [`RRScheduler`]: Round-robin scheduler (preemptive).

#![cfg_attr(not(test), no_std)]
#![feature(const_mut_refs)]

mod zircon_style;

pub use zircon_style::LoadBalanceZirconStyle;

extern crate alloc;

/// The base loadbalance trait that all load balance managers should implement. 
pub trait BaseLoadBalance {
    //type LoadBalanceType;

    /// find target cpu id for a new task
    /// the affinity mask is keep all 1s
    fn find_target_cpu(&self) -> usize;
    
    /// find target cpu id that can be stolen 
    /// the detailed steal process is defined in axtask
    /// >= 0 if a target cpu is found, -1 if no need to steal
    fn find_stolen_cpu_id(&self) -> isize;

    /// add weight
    fn add_weight(&self, delta: isize);
}
