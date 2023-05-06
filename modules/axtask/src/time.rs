use axhal::time::current_time_nanos;

pub struct TimeStat {
    /// 用户态经过的时间，单位为纳秒
    utime_ns: usize,
    /// 内核态经过的时间，单位为纳秒
    stime_ns: usize,
    /// 进入用户态时标记当前时间戳，用于统计用户态时间
    user_tick: usize,
    /// 进入内核态时标记当前时间戳，用于统计内核态时间
    kernel_tick: usize,
}

impl TimeStat {
    /// 新建一个进程时需要初始化时间
    pub fn new() -> Self {
        Self {
            utime_ns: 0,
            stime_ns: 0,
            user_tick: 0,
            // 创建新任务时一般都在内核内，所以可以认为进入内核的时间就是当前时间
            kernel_tick: current_time_nanos() as usize,
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
    pub fn into_kernel_mode(&mut self) {
        let now_time_ns = current_time_nanos() as usize;
        self.utime_ns += now_time_ns - self.user_tick;
        self.kernel_tick = now_time_ns;
    }
    /// 从内核态进入用户态，记录当前时间戳，统计内核态时间
    pub fn into_user_mode(&mut self) {
        // 获取当前时间，单位为纳秒
        let now_time_ns = current_time_nanos() as usize;
        self.stime_ns += now_time_ns - self.kernel_tick;
        self.user_tick = now_time_ns;
    }
    /// 内核态下，当前任务被切换掉，统计内核态时间
    pub fn swtich_from(&mut self) {
        // 获取当前时间，单位为纳秒
        let now_time_ns = current_time_nanos() as usize;
        self.stime_ns += now_time_ns - self.kernel_tick;
        // 需要更新内核态时间戳
        self.kernel_tick = now_time_ns;
    }
    /// 内核态下，切换到当前任务，更新内核态时间戳
    pub fn switch_to(&mut self) {
        // 获取当前时间，单位为纳秒
        self.kernel_tick = current_time_nanos() as usize;
    }
    /// 将时间转化为秒与微秒输出，方便sys_times使用
    pub fn output_as_us(&self) -> (usize, usize, usize, usize) {
        let utime_s = self.utime_ns / 1000_0000_0000;
        let stime_s = self.stime_ns / 1000_0000_0000;
        let utime_us = self.utime_ns / 1000;
        let stime_us = self.stime_ns / 1000;
        (utime_s, stime_s, utime_us, stime_us)
    }
}
