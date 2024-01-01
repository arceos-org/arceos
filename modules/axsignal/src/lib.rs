//! 信号处理模块
//!
//! 当前模型中，进程与线程分离，认为信号模块是进程层面的内容，同一进程下不同线程共享信号处理模块。
//!
//! 当前采用在trap return时进行信号的处理。因此为了防止信号处理延时过长，需要开启时钟中断，使得OS每隔一段时间触发
//! 一次trap，从而检查是否有需要处理的信号。
#![cfg_attr(not(test), no_std)]

use action::SigAction;
use signal_no::{SignalNo, MAX_SIG_NUM};

pub use action::SIGNAL_RETURN_TRAP;
pub mod action;
pub mod info;
pub mod signal_no;
pub mod ucontext;

/// 处理所有信号的结构
#[derive(Clone)]
pub struct SignalHandler {
    pub handlers: [Option<SigAction>; MAX_SIG_NUM],
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalHandler {
    /// 新建一个信号处理函数
    pub fn new() -> Self {
        Self {
            // 默认所有信号都是默认处理
            handlers: [None; MAX_SIG_NUM],
        }
    }

    /// 清空信号处理模块
    ///
    /// 会在exec时清空
    pub fn clear(&mut self) {
        for action in self.handlers.iter_mut() {
            *action = None;
        }
    }

    pub fn get_action(&self, sig_num: usize) -> Option<&SigAction> {
        // 若未设置对应函数或者对应函数为SIG_DFL，则代表默认处理，直接返回空
        self.handlers[sig_num - 1]
            .as_ref()
            .filter(|&action| action.sa_handler != action::SIG_DFL)
    }
    /// 设置信号处理函数
    ///
    /// # Safety
    ///
    /// 传入的action必须是合法的指针
    pub unsafe fn set_action(&mut self, sig_num: usize, action: *const SigAction) {
        self.handlers[sig_num - 1] = Some(unsafe { *action });
    }
}

/// 接受信号的结构，每一个进程都有一个
#[derive(Clone, Copy)]
pub struct SignalSet {
    /// 信号掩码
    pub mask: usize,
    /// 未决信号集
    pub pending: usize,
}

impl Default for SignalSet {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalSet {
    /// 新建处理模块
    pub fn new() -> Self {
        Self {
            mask: 0,
            pending: 0,
        }
    }

    /// 清空信号处理模块
    pub fn clear(&mut self) {
        self.mask = 0;
        self.pending = 0;
    }

    /// 查询是否有未决信号，若有则返回对应编号
    ///
    /// 但是不会修改原有信号集
    pub fn find_signal(&self) -> Option<usize> {
        let mut temp_pending = self.pending;
        loop {
            let pos: u32 = temp_pending.trailing_zeros();
            // 若全为0，则返回64，代表没有未决信号
            if pos == MAX_SIG_NUM as u32 {
                return None;
            } else {
                temp_pending &= !(1 << pos);

                if (self.mask & (1 << pos) == 0)
                    || pos == SignalNo::SIGKILL as u32 - 1
                    || pos == SignalNo::SIGSTOP as u32 - 1
                {
                    break Some(pos as usize + 1);
                }
            }
        }
    }

    /// 查询当前是否有未决信号
    ///
    /// 若有则返回信号编号最低的一个，，并且修改原有信号集
    pub fn get_one_signal(&mut self) -> Option<usize> {
        match self.find_signal() {
            Some(pos) => {
                // 修改原有信号集
                self.pending &= !(1 << (pos - 1));
                Some(pos)
            }
            None => None,
        }
    }

    /// 尝试添加一个bit作为信号
    ///
    /// 若当前信号已经加入到未决信号集中，则不作处理
    ///
    /// 若信号在掩码中，则仍然加入，但是可能不会触发
    pub fn try_add_signal(&mut self, sig_num: usize) {
        let now_mask = 1 << (sig_num - 1);
        self.pending |= now_mask;
    }
}
