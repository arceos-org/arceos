//! 提供一个临时、干净的栈环境，以便判断操作是否触发异常
//!
//! 例：当 backtrace 尝试通过 fp 回溯时，
//! 由于不同内核实现不同，我们无法立即判断当前 fp 位置是否合法。
//! 尽管可以通过查询页表等结构、直接触发内核异常等方式来处理，
//! 但由于 backtrace 通常发生在 panic 时，所以我们不希望引入更多
//! 代码，以此避免连环 panic。
//! 此时，就可以使用另一个极简的 trap_handler 去检查访问是否正常。
//! 这就是 alter_trap 所做的。
//!

#![no_std]

#[cfg(not(target_arch = "riscv64"))]
compile_error!("alter_trap has only impl on riscv64");

extern crate kernel_guard;
extern crate riscv;

mod test_result;
pub use test_result::TestResult;
mod alter_trap_guard;
use alter_trap_guard::AlterTrapGuard;

core::arch::global_asm!(include_str!("trap.S"));

extern "C" {
    fn __alter_trap_read_at(addr: usize, _: usize) -> TestResult;
    fn __alter_trap_write_at(addr: usize, _: usize, value: usize) -> TestResult;
}

/// SAFETY: AlterTrapGuard 实现保证了执行过程中不会有其他异常中断干扰，
/// 因此对外部调用来说，此函数是 safe 的
pub fn alter_trap_read_at(addr: usize) -> TestResult {
    let _guard = AlterTrapGuard::new();
    unsafe { __alter_trap_read_at(addr, 0) }
}

/// SAFETY: AlterTrapGuard 实现保证了执行过程中不会有其他异常中断干扰，
/// 因此对外部调用来说，此函数是 safe 的
pub fn alter_trap_write_at(addr: usize, value: usize) -> TestResult {
    let _guard = AlterTrapGuard::new();
    unsafe { __alter_trap_write_at(addr, 0, value) }
}
