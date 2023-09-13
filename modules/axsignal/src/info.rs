//! 触发信号时的信息，当SigAction指定需要信息时，将其返回给用户
//!
//! 错误信息：详细定义见 `https://man7.org/linux/man-pages/man2/rt_sigaction.2.html`

pub struct SigInfo {
    pub si_signo: i32,
    pub si_errno: i32,
    pub si_code: i32,
}

impl Default for SigInfo {
    fn default() -> Self {
        Self {
            si_signo: 0,
            si_errno: 0,
            si_code: -6, // SI_TKILL
        }
    }
}
