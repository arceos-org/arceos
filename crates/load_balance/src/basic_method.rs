use crate::BaseLoadBalance;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::{AtomicIsize, Ordering};
use log::info;
use spinlock::SpinNoIrq;

/// simply find target CPU and stolen CPU according to mininum payload.
pub struct BasicMethod {
    smp: AtomicUsize,
    /// self queue id
    id: AtomicUsize,
    /// pointers for BaseLoadBalance for each queue
    pointers: SpinNoIrq<Vec<Arc<BasicMethod>>>,
    /// estimated weight for its queue
    weight: AtomicIsize,
}

impl BasicMethod {
    /// Creates a new empty [`BasicMethod`].
    pub const fn new(_id: usize) -> Self {
        Self {
            weight: AtomicIsize::new(0),
            smp: AtomicUsize::new(0),
            id: AtomicUsize::new(0),
            pointers: SpinNoIrq::new(Vec::new()),
        }
    }
    /// get the name of load balance manager
    pub fn load_balance_name() -> &'static str {
        "basic method"
    }
}

impl BasicMethod {
    /// Initializes the load balance manager.
    pub fn init(&self, smp: usize, loadbalancearr: Vec<Arc<BasicMethod>>) {
        self.smp.store(smp, Ordering::Release);
        for i in 0..smp {
            self.pointers.lock().push(loadbalancearr[i].clone());
        }
    }
}

impl BaseLoadBalance for BasicMethod {
    /// the most naive method : find min
    fn find_target_cpu(&self, aff: u64) -> usize {
        let mut mn = 0;
        let mut arg: isize = -1;
        for i in 0..self.smp.load(Ordering::Acquire) {
            if ((aff >> i) & 1) == 1 {
                let tmp = self.pointers.lock()[i].weight.load(Ordering::Acquire);
                if arg == -1 || tmp < mn {
                    mn = tmp;
                    arg = i as isize;
                }
            }
        }
        arg as usize
    }

    /// find target cpu id that can be stolen
    /// the detailed steal process is defined in axtask
    /// >= 0 if a target cpu is found, -1 if no need to steal
    fn find_stolen_cpu_id(&self) -> isize {
        let mut mx: isize = self.pointers.lock()[0].weight.load(Ordering::Acquire);
        let mut arg: usize = 0;
        //info!("qwq");
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
