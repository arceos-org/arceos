//! 信号处理模块
//!
//! 当前模型中，进程与线程分离，认为信号模块是进程层面的内容，同一进程下不同线程共享信号处理模块。
//!
//! 当前采用在trap return时进行信号的处理。因此为了防止信号处理延时过长，需要开启时钟中断，使得OS每隔一段时间触发
//! 一次trap，从而检查是否有需要处理的信号。
#![no_std]

use action::SigAction;
use signal_no::MAX_SIG_NUM;

pub mod action;
pub mod info;
pub mod signal_no;
pub mod ucontext;

/// 处理所有信号的结构
#[derive(Clone)]
pub struct SignalHandler {
    pub handlers: [Option<SigAction>; MAX_SIG_NUM],
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
        if let Some(action) = self.handlers[sig_num - 1].as_ref() {
            if action.sa_handler == action::SIG_DFL {
                return None;
            } else {
                return Some(action);
            }
        } else {
            return None;
        }
    }

    pub fn set_action(&mut self, sig_num: usize, action: *const SigAction) {
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

    /// 查询当前是否有未决信号
    ///
    /// 若有则返回信号编号最低的一个
    pub fn get_one_signal(&mut self) -> Option<usize> {
        let pos: u32 = self.pending.trailing_zeros();
        // 若全为0，则返回64，代表没有未决信号
        if pos == MAX_SIG_NUM as u32 {
            return None;
        } else {
            // 将对应位置改为0代表已经处理
            self.pending &= !(1 << pos);
            return Some(pos as usize + 1);
        }
    }

    /// 尝试添加一个bit作为信号
    ///
    /// 若当前信号已经加入到未决信号集中，则不作处理
    pub fn try_add_signal(&mut self, sig_num: usize) {
        let now_mask = 1 << (sig_num - 1);
        self.pending |= now_mask;
    }
}
