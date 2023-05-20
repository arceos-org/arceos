//! Various scheduler algorithms in a unified interface.
//!
//! Currently supported algorithms:
//!
//! - [`FifoScheduler`]: FIFO (First-In-First-Out) scheduler (cooperative).
//! - [`RRScheduler`]: Round-robin scheduler (preemptive).

use core::sync::atomic::{AtomicIsize, Ordering};
use alloc::vec::Vec;
use alloc::sync::Arc;
use crate::BaseLoadBalance;
use core::sync::atomic::AtomicUsize;
use spinlock::SpinNoIrq;

//use log::debug;

pub struct LoadBalanceZirconStyle {
    smp: AtomicUsize,
    /// self queue id
    id: AtomicUsize,
    /// pointers for BaseLoadBalance for each queue
    pointers: SpinNoIrq<Vec<Arc<LoadBalanceZirconStyle>>>,
    /// estimated weight for its queue
    weight: AtomicIsize,
}

impl LoadBalanceZirconStyle {
    /// Creates a new empty [`LoadBalanceZirconStyle`].
    pub const fn new(id_: usize) -> Self {
        Self {
            weight: AtomicIsize::new(0),
            smp: AtomicUsize::new(0),
            id: AtomicUsize::new(0),
            pointers: SpinNoIrq::new(Vec::new()),
        }
    }
    /// get the name of load balance manager
    pub fn load_balance_name() -> &'static str {
        "zircon style"
    }
}

impl LoadBalanceZirconStyle {
    pub fn init(&self, smp: usize, loadbalancearr: Vec<Arc<LoadBalanceZirconStyle>>) {
        self.smp.store(smp, Ordering::Release);
        for i in 0..smp {
            self.pointers.lock().push(loadbalancearr[i].clone());
        }
    }
}

impl BaseLoadBalance for LoadBalanceZirconStyle {
    //type LoadBalanceType = LoadBalanceZirconStyle;

    /// the most naive method : find min
    fn find_target_cpu(&self) -> usize {
        let mut mn: isize = self.pointers.lock()[0].weight.load(Ordering::Acquire);
        let mut arg: usize = 0;
        for i in 1..self.smp.load(Ordering::Acquire) {
            let tmp = self.pointers.lock()[i].weight.load(Ordering::Acquire);
            if tmp < mn {
                mn = tmp;
                arg = i;
            }
        }
        arg
        //0
    }
    
    /// find target cpu id that can be stolen 
    /// the detailed steal process is defined in axtask
    /// >= 0 if a target cpu is found, -1 if no need to steal
    fn find_stolen_cpu_id(&self) -> isize {
        //debug!("weight: ");
        //for i in 0..self.smp.load(Ordering::Acquire) {
        //    debug!("{}", self.pointers.lock()[i].weight.load(Ordering::Acquire));
        //}
        let mut mx: isize = self.pointers.lock()[0].weight.load(Ordering::Acquire);
        let mut arg: usize = 0;
        for i in 1..self.smp.load(Ordering::Acquire) {
            let tmp = self.pointers.lock()[i].weight.load(Ordering::Acquire);
            if tmp > mx {
                mx = tmp;
                arg = i;
            }
        }
        if mx == 0 || arg == self.id.load(Ordering::Acquire) {
            -1
        } else {
            arg as isize
        }
    }

    fn add_weight(&self, delta: isize) {
        self.weight.fetch_add(delta, Ordering::Release);
    }
    fn get_weight(&self) -> isize {
        self.weight.load(Ordering::Acquire)
    }
}