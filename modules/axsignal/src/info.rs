//! 触发信号时的信息，当SigAction指定需要信息时，将其返回给用户
//!
//! 错误信息：详细定义见 `https://man7.org/linux/man-pages/man2/rt_sigaction.2.html`

/// The information of the signal
///
/// When the `SigAction` specifies that it needs information, it will return it to the user
pub struct SigInfo {
    /// The signal number
    pub si_signo: i32,
    /// An errno value
    pub si_errno: i32,
    /// The code of the signal
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
