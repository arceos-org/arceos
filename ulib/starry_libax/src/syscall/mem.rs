use crate::{fs::FileDesc, syscall::syscall_id::ErrorNo};

use super::flags::{MMAPFlags, MMAPPROT};
extern crate alloc;
use alloc::boxed::Box;
use axmem::MemBackend;
use axprocess::process::current_process;
use log::{debug, info};
use memory_addr::VirtAddr;
const MAX_HEAP_SIZE: usize = 0x20000;
/// 修改用户堆大小，
///
/// - 如输入 brk 为 0 ，则返回堆顶地址
/// - 重新设置堆顶地址，如成功则返回设置后的堆顶地址，否则保持不变，并返回之前的堆顶地址。
pub fn syscall_brk(brk: usize) -> isize {
    let curr_process = current_process();
    let mut inner = curr_process.inner.lock();
    let mut return_val: isize = inner.heap_top as isize;
    if brk != 0 {
        if brk >= inner.heap_bottom && brk <= inner.heap_bottom + MAX_HEAP_SIZE {
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
    offset: usize,
) -> isize {
    debug!(
        "mmap start={:x} len={:x} prot=[{:#?}] flags=[{:#?}] fd={} offset={:x}",
        start, len, prot, flags, fd, offset
    );
    let fixed = flags.contains(MMAPFlags::MAP_FIXED);
    // try to map to NULL
    if fixed && start == 0 {
        return ErrorNo::EINVAL as isize;
    }

    let curr = current_process();
    let inner = curr.inner.lock();
    let addr = if flags.contains(MMAPFlags::MAP_ANONYMOUS) {
        // no file
        if !(fd == -1 && offset == 0) {
            return ErrorNo::EINVAL as isize;
        }
        inner
            .memory_set
            .lock()
            .mmap(start.into(), len, prot.into(), fixed, None)
    } else {
        // file backend
        debug!("[mmap] fd: {}, offset: 0x{:x}", fd, offset);
        if fd >= inner.fd_manager.fd_table.len() as i32 || fd < 0 {
            return ErrorNo::EINVAL as isize;
        }
        let file = match &inner.fd_manager.fd_table[fd as usize] {
            // 文件描述符表里面存的是文件描述符，这很合理罢
            Some(file) => Box::new(
                file.lock()
                    .as_any()
                    .downcast_ref::<FileDesc>()
                    .expect("Try to mmap with a non-file backend")
                    .file
                    .lock()
                    .clone(),
            ),
            // fd not found
            None => return ErrorNo::EINVAL as isize,
        };

        let backend = MemBackend::new(file, offset as u64);
        inner
            .memory_set
            .lock()
            .mmap(start.into(), len, prot.into(), fixed, Some(backend))
    };
    drop(inner);
    drop(curr);

    unsafe { riscv::asm::sfence_vma_all() };
    debug!("mmap: 0x{:x}", addr);
    // info!("val: {}", unsafe { *(addr as *const usize) });
    addr
}

pub fn syscall_munmap(start: usize, len: usize) -> isize {
    let curr = current_process();
    let inner = curr.inner.lock();
    inner.memory_set.lock().munmap(start.into(), len);
    drop(inner);
    drop(curr);
    unsafe { riscv::asm::sfence_vma_all() };

    0
}

pub fn syscall_msync(start: usize, len: usize) -> isize {
    let curr = current_process();
    let inner = curr.inner.lock();
    info!("test");
    inner.memory_set.lock().msync(start.into(), len);

    drop(inner);
    drop(curr);

    0
}

pub fn syscall_mprotect(start: usize, len: usize, prot: MMAPPROT) -> isize {
    let curr = current_process();
    let inner = curr.inner.lock();

    inner
        .memory_set
        .lock()
        .mprotect(VirtAddr::from(start), len, prot.into());

    unsafe { riscv::asm::sfence_vma_all() };

    0
}
