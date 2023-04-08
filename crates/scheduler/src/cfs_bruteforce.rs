use alloc::{collections::VecDeque, sync::Arc};
use core::ops::Deref;
use core::sync::atomic::{AtomicIsize, Ordering};

use crate::BaseScheduler;

pub struct CFTask<T> {
    inner: T,
    init_vruntime: AtomicIsize,
    delta: AtomicIsize,
    nice: AtomicIsize,
}

// TODO：现在全部都是暴力实现

const NICE2WEIGHT_POS: [isize; 20] = [1024, 820, 655, 526, 423, 335, 272, 215, 172, 137, 110, 87, 70, 56, 45, 36, 29, 23, 18, 15];
const NICE2WEIGHT_NEG: [isize; 21] = [1024, 1277, 1586, 1991, 2501, 3121, 3906, 4904, 6100, 7620, 9548, 11916, 14949, 18705, 23254, 29154, 36291, 46273, 56483, 71755, 88761];

impl<T> CFTask<T> {
    pub const fn new(inner: T, n: isize) -> Self {
        Self {
            inner,
            init_vruntime: AtomicIsize::new(0 as isize),
            delta: AtomicIsize::new(0 as isize),
            nice: AtomicIsize::new(n as isize),
        }
    }
    
    fn get_weight(&self) -> isize {
        if self.nice.load(Ordering::Acquire) >= 0 {
            NICE2WEIGHT_POS[self.nice.load(Ordering::Acquire) as usize]
        } else {
            NICE2WEIGHT_NEG[(-self.nice.load(Ordering::Acquire)) as usize]
        }
    }

    pub fn get_vruntime(&self) -> isize {
        if self.nice.load(Ordering::Acquire) == 0 {
            self.init_vruntime.load(Ordering::Acquire) + self.delta.load(Ordering::Acquire)
        }
        else {
            self.init_vruntime.load(Ordering::Acquire) + self.delta.load(Ordering::Acquire) * 1024 / self.get_weight()
        }
    }

    pub fn set_vruntime(&self, v : isize) {
        self.init_vruntime.store(v, Ordering::Release);
    }

    pub fn task_tick(&self) -> isize {
        let d = self.delta.load(Ordering::Acquire);
        self.delta.store(d + 1, Ordering::Release);
        self.get_vruntime()
    }

    pub const fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T> const Deref for CFTask<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct CFScheduler<T> {
    ready_queue: VecDeque<Arc<CFTask<T>>>,
    min_vruntime: Option<AtomicIsize>,
}

impl<T> CFScheduler<T> {
    pub const fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            min_vruntime: None
        }
    }
}
 
impl<T> BaseScheduler for CFScheduler<T> {
    type SchedItem = Arc<CFTask<T>>;

    fn init(&mut self) {}

    fn add_task(&mut self, task: Self::SchedItem) {
        if !self.min_vruntime.is_some() {
            self.min_vruntime = Some(AtomicIsize::new(0 as isize));
        }
        (*task).set_vruntime(self.min_vruntime.as_mut().unwrap().load(Ordering::Acquire));
        self.ready_queue.push_back(task);
    }

    fn remove_task(&mut self, task: &Self::SchedItem) -> Option<Self::SchedItem> {
        // TODO: more efficient
        self.ready_queue
            .iter()
            .position(|t| Arc::ptr_eq(t, task))
            .and_then(|idx| self.ready_queue.remove(idx))
    }

    fn pick_next_task(&mut self) -> Option<Self::SchedItem> {
        // 待加速
        let mut mn = 0;
        let mut arg = 0;
        for i in 0..self.ready_queue.len() {
            let vruntime = self.ready_queue[i].get_vruntime();
            if i == 0 || vruntime < mn {
                mn = vruntime;
                arg = i;
            }
        }
        if self.ready_queue.len() > 0 {
            self.ready_queue.swap(0, arg);
        }
        self.ready_queue.pop_front()
    }

    fn put_prev_task(&mut self, prev: Self::SchedItem, _preempt: bool) {
        // TODO: 现在还不支持 preempt，现在还在研究内核是怎么写的
        self.ready_queue.push_back(prev)
    }

    fn task_tick(&mut self, current: &Self::SchedItem) -> bool {
        let curr_vruntime = current.task_tick();
        let mut mn = curr_vruntime;
        for i in 0..self.ready_queue.len() {
            let vruntime = self.ready_queue[i].get_vruntime();
            if vruntime < mn {
                mn = vruntime;
            }
        }
        self.min_vruntime.as_mut().unwrap().store(mn, Ordering::Release);
        mn == curr_vruntime
    }
}
