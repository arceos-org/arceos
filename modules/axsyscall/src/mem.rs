use alloc::sync::Arc;
use axprocess::process::{MAX_HEAP_SIZE, PID2PC};
use axtask::current;

/// 修改用户堆大小，
///
/// - 如输入 brk 为 0 ，则返回堆顶地址
/// - 否则，尝试修改堆顶为 brk，成功时返回0，失败时返回-1。
pub fn syscall_brk(brk: usize) -> isize {
    let curr = current();
    let pid2pc_inner = PID2PC.lock();
    let curr_process = Arc::clone(&pid2pc_inner.get(&curr.get_process_id()).unwrap());
    drop(pid2pc_inner);
    let mut inner = curr_process.inner.lock();
    let mut return_val: isize = inner.heap_top as isize;
    if brk != 0 {
        if brk < inner.heap_bottom || brk > inner.heap_bottom + MAX_HEAP_SIZE {
            return_val = -1;
        } else {
            inner.heap_top = brk;
            return_val = brk as isize;
        }
    }
    drop(inner);
    return_val
}
