//! AlterTrapGuard:
//!
//! 在这个 struct 存在过程中，切换 trap_handler 至 __alter_trap
//!
//! 为了保证此过程中不被其他异常中断干扰，在切换前后会提前关闭中断

use kernel_guard::{BaseGuard, NoPreemptIrqSave};
use riscv::register::{stvec, stvec::Stvec};

pub struct AlterTrapGuard {
    old_stvec: Stvec,
    _irq_guard: NoPreemptIrqSave,
}

extern "C" {
    fn __alter_trap_entry();
}

impl BaseGuard for AlterTrapGuard {
    /// The saved state when entering the critical section.
    type State = Stvec;

    /// Something that must be done before entering the critical section.
    fn acquire() -> Self::State {
        let old = stvec::read();
        unsafe {
            stvec::write(__alter_trap_entry as usize, stvec::TrapMode::Direct);
        }
        old
    }

    /// Something that must be done after leaving the critical section.
    fn release(state: Self::State) {
        unsafe {
            stvec::write(state.address(), state.trap_mode().unwrap());
        }
    }
}

impl AlterTrapGuard {
    /// Creates a new [`AlterTrapGuard`] guard.
    pub fn new() -> Self {
        // get irq_guard first
        let irq_guard = NoPreemptIrqSave::new();
        Self {
            old_stvec: Self::acquire(),
            _irq_guard: irq_guard,
        }
    }
}

impl Drop for AlterTrapGuard {
    fn drop(&mut self) {
        Self::release(self.old_stvec)
        // irq_guard dropped here
    }
}

impl Default for AlterTrapGuard {
    fn default() -> Self {
        Self::new()
    }
}
