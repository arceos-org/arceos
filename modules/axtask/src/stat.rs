//! 负责任务时间统计的实现
use axhal::time::{current_time_nanos, NANOS_PER_MICROS, NANOS_PER_SEC};
#[cfg(feature = "signal")]
use axsignal::signal_no::SignalNo;
#[cfg(feature = "signal")]
use crate_interface::{call_interface, def_interface};
numeric_enum_macro::numeric_enum! {
    #[repr(i32)]
    #[allow(non_camel_case_types)]
    #[derive(Eq, PartialEq, Debug, Clone, Copy)]
    /// sys_settimer / sys_gettimer 中设定的 which，即计时器类型
    pub enum TimerType {
        /// 表示目前没有任何计时器(不在linux规范中，是os自己规定的)
        NONE = -1,
        /// 统计系统实际运行时间
        REAL = 0,
        /// 统计用户态运行时间
        VIRTUAL = 1,
        /// 统计进程的所有用户态/内核态运行时间
        PROF = 2,
    }
}

impl From<usize> for TimerType {
    fn from(num: usize) -> Self {
        match Self::try_from(num as i32) {
            Ok(val) => val,
            Err(_) => Self::NONE,
        }
    }
}
pub struct TimeStat {
    /// 用户态经过的时间，单位为纳秒
    utime_ns: usize,
    /// 内核态经过的时间，单位为纳秒
    stime_ns: usize,
    /// 进入用户态时标记当前时间戳，用于统计用户态时间
    user_tick: usize,
    /// 进入内核态时标记当前时间戳，用于统计内核态时间
    kernel_tick: usize,
    /// 计时器类型
    timer_type: TimerType,
    /// 设置下一次触发计时器的区间
    /// 当 timer_remained_us 归零时，**如果 timer_interval_us 非零**，则将其重置为 timer_interval_us 的值；
    /// 否则，则这个计时器不再触发
    timer_interval_ns: usize,
    /// 当前轮次下计数器剩余的时间
    ///
    /// 根据timer_type的种类来进行计算，当归零的时候触发信号，同时进行更新
    timer_remained_ns: usize,
}

#[cfg(feature = "signal")]
#[def_interface]
pub trait SignalCaller {
    /// Handles interrupt requests for the given IRQ number.
    fn send_signal(tid: isize, signum: isize);
}

#[allow(unused)]
impl TimeStat {
    /// 新建一个进程时需要初始化时间
    pub fn new() -> Self {
        Self {
            utime_ns: 0,
            stime_ns: 0,
            user_tick: 0,
            // 创建新任务时一般都在内核内，所以可以认为进入内核的时间就是当前时间
            kernel_tick: current_time_nanos() as usize,
            timer_type: TimerType::NONE,
            timer_interval_ns: 0,
            timer_remained_ns: 0,
        }
    }
    /// 清空时间统计，用于exec
    pub fn clear(&mut self) {
        self.utime_ns = 0;
        self.stime_ns = 0;
        self.user_tick = 0;
        self.kernel_tick = current_time_nanos() as usize;
    }
    /// 从用户态进入内核态，记录当前时间戳，统计用户态时间
    pub fn into_kernel_mode(&mut self, tid: isize) {
        let now_time_ns = current_time_nanos() as usize;
        let delta = now_time_ns - self.user_tick;
        self.utime_ns += delta;
        self.kernel_tick = now_time_ns;
        if self.timer_type != TimerType::NONE {
            self.update_timer(delta, tid);
        };
    }
    /// 从内核态进入用户态，记录当前时间戳，统计内核态时间
    pub fn into_user_mode(&mut self, tid: isize) {
        // 获取当前时间，单位为纳秒
        let now_time_ns = current_time_nanos() as usize;
        let delta = now_time_ns - self.kernel_tick;
        self.stime_ns += delta;
        self.user_tick = now_time_ns;
        if self.timer_type == TimerType::REAL || self.timer_type == TimerType::PROF {
            self.update_timer(delta, tid);
        };
    }
    /// 内核态下，当前任务被切换掉，统计内核态时间
    pub fn swtich_from(&mut self, tid: isize) {
        // 获取当前时间，单位为纳秒
        let now_time_ns = current_time_nanos() as usize;
        let delta = now_time_ns - self.kernel_tick;
        self.stime_ns += delta;
        // 需要更新内核态时间戳
        self.kernel_tick = now_time_ns;
        if self.timer_type == TimerType::REAL || self.timer_type == TimerType::PROF {
            self.update_timer(delta, tid);
        };
    }
    /// 内核态下，切换到当前任务，更新内核态时间戳
    pub fn switch_to(&mut self, tid: isize) {
        // 获取当前时间，单位为纳秒
        let now_time_ns = current_time_nanos() as usize;
        let delta = now_time_ns - self.kernel_tick;
        // 更新时间戳，方便当该任务被切换时统计内核经过的时间
        self.kernel_tick = now_time_ns;
        // 注意，对于REAL类型的任务，此时也需要统计经过的时间
        if self.timer_type == TimerType::REAL {
            self.update_timer(delta, tid)
        }
    }
    /// 将时间转化为秒与微秒输出，方便sys_times使用
    /// (用户态秒，用户态微妙，内核态秒，内核态微妙)
    pub fn output_as_us(&self) -> (usize, usize, usize, usize) {
        let utime_s = self.utime_ns / (NANOS_PER_SEC as usize);
        let stime_s = self.stime_ns / (NANOS_PER_SEC as usize);
        let utime_us = self.utime_ns / (NANOS_PER_MICROS as usize);
        let stime_us = self.stime_ns / (NANOS_PER_MICROS as usize);
        (utime_s, utime_us, stime_s, stime_us)
    }

    /// 以微秒形式输出计时器信息
    ///
    /// (计时器周期，当前计时器剩余时间)
    pub fn output_timer_as_us(&self) -> (usize, usize) {
        (self.timer_interval_ns / 1000, self.timer_remained_ns / 1000)
    }

    /// 设定计时器信息
    ///
    /// 若type不为None则返回成功
    pub fn set_timer(
        &mut self,
        timer_interval_ns: usize,
        timer_remained_ns: usize,
        timer_type: usize,
    ) -> bool {
        self.timer_type = timer_type.into();
        self.timer_interval_ns = timer_interval_ns;
        self.timer_remained_ns = timer_remained_ns;
        self.timer_type != TimerType::NONE
    }

    /// 更新计时器，同时判断是否要发出信号
    pub fn update_timer(&mut self, delta: usize, _tid: isize) {
        if self.timer_remained_ns == 0 {
            // 计时器已经结束了
            return;
        }
        if self.timer_remained_ns > delta {
            // 此时计时器还没有结束，直接更新其内容
            self.timer_remained_ns -= delta;
            return;
        }
        // 此时计时器已经结束了，需要进行重置
        self.timer_remained_ns = self.timer_interval_ns;

        #[cfg(feature = "signal")]
        {
            let signal_num = match &self.timer_type {
                TimerType::REAL => SignalNo::SIGALRM,
                TimerType::VIRTUAL => SignalNo::SIGVTALRM,
                TimerType::PROF => SignalNo::SIGPROF,
                _ => SignalNo::ERR,
            };
            if signal_num != SignalNo::ERR {
                call_interface!(SignalCaller::send_signal(_tid, signal_num as isize));
            }
        }
    }
}
