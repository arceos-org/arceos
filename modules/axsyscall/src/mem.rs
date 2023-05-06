use crate::flags::{MMAPFlags, MMAPPROT};
use alloc::sync::Arc;
use alloc::vec;
use axfs_os::read_file_with_offset;
use axprocess::process::current_process;
use memory_addr::{align_down_4k, align_up_4k};
const MAX_HEAP_SIZE: usize = 4096;
/// 修改用户堆大小，
///
/// - 如输入 brk 为 0 ，则返回堆顶地址
/// - 否则，尝试修改堆顶为 brk，成功时返回0，失败时返回-1。
pub fn syscall_brk(brk: usize) -> isize {
    let curr_process = current_process();
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

/// 将文件内容映射到内存中
/// offset参数指定了从文件区域中的哪个字节开始映射，它必须是系统分页大小的倍数
/// len指定了映射文件的长度
/// prot指定了页面的权限
/// flags指定了映射的方法
pub fn syscall_mmap(
    start: usize,
    len: usize,
    prot: MMAPPROT,
    flags: MMAPFlags,
    fd: i32,
    offest: usize,
) -> isize {
    let len = align_up_4k(start + len);
    let start = align_down_4k(start);
    // start为0代表自动分配起始地址
    // 不可以与MMAP_FIXED同时使用
    if start == 0 && flags.contains(MMAPFlags::MAP_FIXED) {
        return -1;
    }
    let random_pos = start == 0 || !flags.contains(MMAPFlags::MAP_FIXED);
    let curr_process = current_process();
    let mut inner = curr_process.inner.lock();
    // 若是不要求实际映射到文件，则只需要为其分配一段物理空间即可
    if flags.contains(MMAPFlags::MAP_ANONYMOUS) {
        // 此时应当要求fd = -1, offset = 0。
        if fd == -1 && offest == 0 {
            inner
                .memory_set
                .lock()
                .mmap(start.into(), len, prot.into(), random_pos, None)
        } else {
            drop(inner);
            -1
        }
    } else {
        if fd as usize >= inner.fd_table.len() || fd < 0 || inner.fd_table[fd as usize].is_none() {
            drop(inner);
            return -1;
        }
        // 为了进行映射，有以下几个步骤
        // 一是读取文件内容，由于我们未实现懒分配，所以map时要把文件实际内容写入到物理页面中
        // 二是为文件内容分配物理页面，若是任意寻找位置，则直接找一个大小适合的连续物理页面放进去即可
        // 若是固定位置，则需要在固定位置处进行解映射，然后再进行映射。这个过程需要检查是否越界
        if let Some(file) = &inner.fd_table[fd as usize] {
            file.seek(offest);
            // 获取文件数据
            let mut data = vec![0u8; len];
            file.read(&mut data);
            inner
                .memory_set
                .lock()
                .mmap(start.into(), len, prot.into(), random_pos, Some(&data))
        } else {
            return -1;
        }
    }
}

pub fn syscall_munmap(start: usize, len: usize) -> isize {
    let len = align_up_4k(start + len);
    let start = align_down_4k(start);
    let curr_process = current_process();
    let mut inner = curr_process.inner.lock();
    let answer = inner.memory_set.lock().munmap(start.into(), len);
    drop(inner);
    answer
}
