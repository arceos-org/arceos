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
    fn __alter_trap_read_usize(addr: usize, _: usize) -> TestResult;
    fn __alter_trap_write_usize(addr: usize, _: usize, value: usize) -> TestResult;
    fn __alter_trap_read_write_usize(addr: usize, _: usize) -> TestResult;
}

/// 测试 addr 是否可读。
/// 返回读取到的值或错误信息，包装在 [TestResult](crate::test_result::TestResult)
/// 
/// 例：
/// ```ignore
/// let value = alter_trap_read_usize(addr).as_option()?;
/// ```
/// 
/// SAFETY: AlterTrapGuard 实现保证了执行过程中不会有其他异常中断干扰，
/// 因此对外部调用来说，此函数是 safe 的
pub fn alter_trap_read_usize(addr: usize) -> TestResult {
    let _guard = AlterTrapGuard::new();
    unsafe { __alter_trap_read_usize(addr, 0) }
}

/// 测试 addr 是否可写。
/// 如不可写则返回错误信息，包装在 [TestResult](crate::test_result::TestResult)
/// 
/// 例：
/// ```ignore
/// let write_ok = alter_trap_write_usize(addr, value).as_bool();
/// // or you can
/// match alter_trap_write_usize(addr, value).as_result() {
///     Ok(_) => {},
///     Err(scause) => {}
/// }
/// ```
/// 
/// 尝试写入 addr，返回一个 [TestResult](crate::test_result::TestResult)
/// SAFETY: AlterTrapGuard 实现保证了执行过程中不会有其他异常中断干扰，
/// 因此对外部调用来说，此函数是 safe 的
pub fn alter_trap_write_usize(addr: usize, value: usize) -> TestResult {
    let _guard = AlterTrapGuard::new();
    unsafe { __alter_trap_write_usize(addr, 0, value) }
}

/// 测试 addr 是否可读可写。
/// 返回读取到的值或错误信息，包装在 [TestResult](crate::test_result::TestResult)
/// 
/// 读取/写入错误可以通过具体错误信息分辨，见下。
/// 
/// 例：
/// ```ignore
/// let value = alter_trap_read_write_usize(addr).as_option()?;
/// // or you can
/// let value = match alter_trap_read_write_usize(addr).as_result() {
///     Ok(v) => {v},
///     Err(TestResult::LOAD_PAGE_FAULT) => {panic!("read failed");},
///     Err(TestResult::STORE_PAGE_FAULT) => {panic!("write failed");},
///     _ => {panic!("Oops the crate itself failed");},
/// };
/// ```
/// 
/// 尝试写入 addr，返回一个 [TestResult](crate::test_result::TestResult)
/// SAFETY: AlterTrapGuard 实现保证了执行过程中不会有其他异常中断干扰，
/// 因此对外部调用来说，此函数是 safe 的
pub fn alter_trap_read_write_usize(addr: usize) -> TestResult {
    //let _guard = black_box(AlterTrapGuard::new());
    let _guard = AlterTrapGuard::new();
    unsafe { __alter_trap_read_write_usize(addr, 0) }
}
