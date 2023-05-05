use bitflags::*;

bitflags! {
    /// 指定 sys_wait4 的选项
    pub struct WaitFlags: u32 {
        /// 不挂起当前进程，直接返回
        const WNOHANG = 1 << 0;
        /// 报告已执行结束的用户进程的状态
        const WIMTRACED = 1 << 1;
        /// 报告还未结束的用户进程的状态
        const WCONTINUED = 1 << 3;
    }
}

#[derive(Clone, Copy)]
pub struct TimeSecs {
    pub tv_sec: usize,  /* 秒 */
    pub tv_nsec: usize, /* 纳秒, 范围在0~999999999 */
}
