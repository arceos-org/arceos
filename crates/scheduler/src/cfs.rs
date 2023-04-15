use alloc::{collections::BTreeMap, sync::Arc};
use core::ops::Deref;
use core::sync::atomic::{AtomicIsize, Ordering};

use crate::BaseScheduler;

pub struct CFTask<T> {
    inner: T,
    init_vruntime: AtomicIsize,
    delta: AtomicIsize,
    nice: AtomicIsize,
    id: AtomicIsize,
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
            id: AtomicIsize::new(0 as isize)
        }
    }
    
    fn get_weight(&self) -> isize {
        if self.nice.load(Ordering::Acquire) >= 0 {
            NICE2WEIGHT_POS[self.nice.load(Ordering::Acquire) as usize]
        } else {
            NICE2WEIGHT_NEG[(-self.nice.load(Ordering::Acquire)) as usize]
        }
    }

    fn get_id(&self) -> isize {
        self.id.load(Ordering::Acquire)
    }

    pub fn get_vruntime(&self) -> isize {
        if self.nice.load(Ordering::Acquire) == 0 {
            self.init_vruntime.load(Ordering::Acquire) + self.delta.load(Ordering::Acquire)
        }
        else {
            self.init_vruntime.load(Ordering::Acquire) + self.delta.load(Ordering::Acquire) * 1024 / self.get_weight()
        }
    }

    pub fn set_vruntime(&self, v: isize) {
        self.init_vruntime.store(v, Ordering::Release);
    }

    pub fn set_id(&self, id: isize) {
        self.id.store(id, Ordering::Release);
    }

    pub fn task_tick(&self) -> isize {
        self.delta.fetch_add(1, Ordering::Release);
        self.get_vruntime()
    }

    pub const fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T> Ord for CFTask<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let a = self.get_vruntime();
        let b = other.get_vruntime();
        let a_id = self.get_id();
        let b_id = other.get_id();
        if a < b {
            core::cmp::Ordering::Less
        } else if a > b {
            core::cmp::Ordering::Greater
        } else if a_id < b_id {
            core::cmp::Ordering::Less
        } else if a_id > b_id {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    }
}

impl<T> Eq for CFTask<T> {}

impl<T> PartialOrd for CFTask<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let a = self.get_vruntime();
        let b = other.get_vruntime();
        let a_id = self.get_id();
        let b_id = other.get_id();
        if a < b {
            Some(core::cmp::Ordering::Less)
        } else if a > b {
            Some(core::cmp::Ordering::Greater)
        } else if a_id < b_id {
            Some(core::cmp::Ordering::Less)
        } else if a_id > b_id {
            Some(core::cmp::Ordering::Greater)
        } else {
            Some(core::cmp::Ordering::Equal)
        }
    }
}

impl<T> PartialEq for CFTask<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_vruntime() == other.get_vruntime() 
    }
}

impl<T> const Deref for CFTask<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct CFScheduler<T> {
    ready_queue: BTreeMap<Arc<CFTask<T>>, isize>,
    min_vruntime: Option<AtomicIsize>,
    id_pool: AtomicIsize
}

impl<T> CFScheduler<T> {
    pub const fn new() -> Self {
        Self {
            ready_queue: BTreeMap::new(),
            min_vruntime: None,
            id_pool: AtomicIsize::new(0 as isize)
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
        (*task).set_id(self.id_pool.fetch_add(1, Ordering::Release));
        self.ready_queue.insert(task, 0);
        if let Some((min_vruntime_task, _)) = self.ready_queue.first_key_value() {
            // TODO: None
            self.min_vruntime = Some(AtomicIsize::new(min_vruntime_task.get_vruntime() as isize));
        } else {
            self.min_vruntime = None;
        }
    }

    fn remove_task(&mut self, task: &Self::SchedItem) -> Option<Self::SchedItem> {
        if let Some(tmp) = self.ready_queue.remove_entry(task) {
            if let Some((min_vruntime_task, _)) = self.ready_queue.first_key_value() {
                // TODO: None
                self.min_vruntime = Some(AtomicIsize::new(min_vruntime_task.get_vruntime() as isize));
            } else {
                self.min_vruntime = None;
            }
            Some(tmp.0)
        } else {
            None
        }
    }

    fn pick_next_task(&mut self) -> Option<Self::SchedItem> {
        if let Some((k, _)) = self.ready_queue.pop_first() {
            Some(k)
        } else {
            None
        }
    }

    fn put_prev_task(&mut self, prev: Self::SchedItem, _preempt: bool) {
        // TODO: 现在还不支持 preempt，现在还在研究内核是怎么写的
        let vruntime = Arc::clone(&prev).get_vruntime();
        self.ready_queue.insert(prev, vruntime);
    }

    fn task_tick(&mut self, _current: &Self::SchedItem) -> bool {
        _current.task_tick();
        !self.min_vruntime.is_some() || _current.get_vruntime() > self.min_vruntime.as_mut().unwrap().load(Ordering::Acquire)
    }
}