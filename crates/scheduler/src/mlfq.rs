use alloc::{collections::VecDeque, sync::Arc, vec::Vec};
use core::ops::Deref;
use core::sync::atomic::{AtomicIsize, Ordering};

use crate::BaseScheduler;

pub struct MLFQTask<T, const QNUM: usize, const BASETICK: usize, const RESETTICK: usize> {
    inner: T,
    prio: AtomicIsize, // 注意值越大优先级越低，每增加 1 相对应的时间片数乘以 2。
    remain_ticks: AtomicIsize,
}

impl<T, const QNUM: usize, const BASETICK: usize, const RESETTICK: usize>
    MLFQTask<T, QNUM, BASETICK, RESETTICK>
{
    pub const fn new(inner: T) -> Self {
        Self {
            inner,
            prio: AtomicIsize::new(0 as isize),
            remain_ticks: AtomicIsize::new(BASETICK as isize),
        }
    }

    pub fn get_prio(&self) -> isize {
        self.prio.load(Ordering::Acquire)
    }

    pub fn tick(&self) -> isize {
        self.remain_ticks.fetch_sub(1, Ordering::Release)
    }

    pub fn get_remain(&self) -> isize {
        self.remain_ticks.load(Ordering::Acquire)
    }

    pub fn reset_ticks(&self) {
        self.remain_ticks.store(
            (BASETICK as isize) << self.prio.load(Ordering::Acquire),
            Ordering::Release,
        );
    }

    // 所有的任务重置优先级
    pub fn reset_prio(&self) {
        self.prio.store(0, Ordering::Release);
        self.remain_ticks
            .store(BASETICK as isize, Ordering::Release);
    }

    // 优先级减一（代码取反了所以是加一，不想用负数），相应设置 remain_ticks。返回的是新的优先级。
    // 注意处理优先级已经到最低的情况。
    pub fn prio_promote(&self) -> isize {
        let mut current_prio = self.prio.fetch_add(1, Ordering::Release) + 1;
        if current_prio == QNUM as isize {
            self.prio.store(QNUM as isize - 1, Ordering::Release);
            current_prio = QNUM as isize - 1;
        }
        self.remain_ticks
            .store((BASETICK as isize) << current_prio, Ordering::Release);
        current_prio as isize
    }

    pub const fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T, const QNUM: usize, const BASETICK: usize, const RESETTICK: usize> Deref
    for MLFQTask<T, QNUM, BASETICK, RESETTICK>
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// 含义分别是：队列个数，基础时间片的滴答数，重置时间片数
pub struct MLFQScheduler<T, const QNUM: usize, const BASETICK: usize, const RESETTICK: usize> {
    ready_queue: Vec<VecDeque<Arc<MLFQTask<T, QNUM, BASETICK, RESETTICK>>>>,
    reset_remain_ticks: AtomicIsize,
}

impl<T, const QNUM: usize, const BASETICK: usize, const RESETTICK: usize>
    MLFQScheduler<T, QNUM, BASETICK, RESETTICK>
{
    pub fn new() -> Self {
        assert!(QNUM > 0);
        let mut ready_queue = Vec::new();
        for _i in 0..QNUM {
            ready_queue.push(VecDeque::new());
        }
        Self {
            ready_queue,
            reset_remain_ticks: AtomicIsize::new(RESETTICK as isize),
        }
    }

    pub fn scheduler_name() -> &'static str {
        "MLFQ"
    }
}

impl<T, const QNUM: usize, const BASETICK: usize, const RESETTICK: usize> BaseScheduler
    for MLFQScheduler<T, QNUM, BASETICK, RESETTICK>
{
    type SchedItem = Arc<MLFQTask<T, QNUM, BASETICK, RESETTICK>>;

    fn init(&mut self) {}

    fn add_task(&mut self, task: Self::SchedItem) {
        self.ready_queue[task.get_prio() as usize].push_back(task);
    }

    fn remove_task(&mut self, task: &Self::SchedItem) -> Option<Self::SchedItem> {
        self.ready_queue[task.get_prio() as usize]
            .iter()
            .position(|t| Arc::ptr_eq(t, task))
            .and_then(|idx| self.ready_queue[task.get_prio() as usize].remove(idx))
    }

    fn pick_next_task(&mut self) -> Option<Self::SchedItem> {
        for i in 0..self.ready_queue.len() {
            if !self.ready_queue[i].is_empty() {
                //info!("pick: {}", self.ready_queue[i][0].get_prio());
                return self.ready_queue[i].pop_front();
            }
        }
        return None;
    }

    fn put_prev_task(&mut self, prev: Self::SchedItem, preempt: bool) {
        // 这个算法只支持部分的 preempt：处于同优先级内可以给它安排在最前面的，但如果优先级不一样就不太行了。这里的 preempt 沿用了 RR 的写法
        if Arc::clone(&prev).get_remain() <= 0 {
            //info!("{}", Arc::clone(&prev).get_prio());
            prev.prio_promote();
            self.ready_queue[Arc::clone(&prev).get_prio() as usize].push_back(prev);
        } else if preempt {
            //info!("={}", Arc::clone(&prev).get_prio());
            prev.reset_ticks();
            self.ready_queue[Arc::clone(&prev).get_prio() as usize].push_front(prev);
        } else {
            //info!("x{}", Arc::clone(&prev).get_prio());
            prev.reset_ticks();
            self.ready_queue[Arc::clone(&prev).get_prio() as usize].push_back(prev);
        }
    }

    fn task_tick(&mut self, _current: &Self::SchedItem) -> bool {
        if self.reset_remain_ticks.fetch_sub(1, Ordering::Release) <= 1 {
            // 触发重启
            self.reset_remain_ticks
                .store(RESETTICK as isize, Ordering::Release);
            let mut new_queue: VecDeque<Arc<MLFQTask<T, QNUM, BASETICK, RESETTICK>>> =
                VecDeque::new();
            // 把所有的任务转移到一个新的 Deque 中
            for i in 0..QNUM {
                while !self.ready_queue[i].is_empty() {
                    new_queue.push_back(self.ready_queue[i].pop_front().unwrap());
                }
            }
            // 重置所有任务的等级和时间片
            // 把这些任务全部塞回 0 号队列中
            let _ = new_queue.iter().map(|item| {
                item.reset_prio();
                self.ready_queue[0].push_back(Arc::clone(item));
            });
            drop(new_queue);
            // 把在外边流浪的 _current 也重置一下
            _current.reset_prio();
            return true; // TODO: 不确定这里的逻辑
        }
        //info!("tick{}", _current.get_remain());
        _current.tick() <= 1
    }

    fn set_priority(&mut self, _task: &Self::SchedItem, _prio: isize) -> bool {
        // pass
        false
    }

    fn is_empty(&self) -> bool {
        for i in 0..QNUM {
            if !self.ready_queue[i].is_empty() {
                return false;
            }
        }
        return true;
    }
}

//std::thread::sleep(Duration::from_millis(10)); // sleep 10 ms
