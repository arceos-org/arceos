extern crate alloc;
use alloc::{sync::Arc, vec::Vec};
use axfs::monolithic_fs::FileIO;
use axprocess::process::{current_process, yield_now_task};
use memory_addr::VirtAddr;
use spinlock::SpinNoIrq;

use super::{flags::TimeSecs, syscall_id::ErrorNo};

// 定义一个bitset，用于查找掩码
struct ShadowBitset {
    addr: *mut usize,
    len: usize, // 是包含的bit数目，而不是字节数目
}

impl ShadowBitset {
    pub fn new(addr: *mut usize, len: usize) -> Self {
        Self { addr, len }
    }

    pub fn check(&self, index: usize) -> bool {
        if index >= self.len {
            return false;
        }
        // 因为一次add会移动八个字节，所以这里需要除以64，即8个字节，每一个字节8位
        let byte_index = index >> 8;
        let bit_index = index & 0x3f;
        unsafe { *self.addr.add(byte_index) & (1 << bit_index) != 0 }
    }

    pub fn set(&mut self, index: usize) {
        if index >= self.len {
            return;
        }
        let byte_index = index >> 8;
        let bit_index = index & 0x3f;
        unsafe {
            *self.addr.add(byte_index) |= 1 << bit_index;
        }
    }

    // 清空自己
    pub fn clear(&self) {
        for i in 0..=(self.len - 1) / 64 {
            unsafe {
                *(self.addr.add(i)) = 0;
            }
        }
    }

    pub fn valid(&self) -> bool {
        self.addr as usize != 0
    }
}

/// 根据给定的地址和长度新建一个fd set，包括文件描述符指针数组，文件描述符数值数组，以及一个bitset
fn init_fd_set(
    addr: *mut usize,
    len: usize,
) -> Result<(Vec<Arc<SpinNoIrq<dyn FileIO>>>, Vec<usize>, ShadowBitset), isize> {
    let process = current_process();
    let inner = process.inner.lock();
    if len >= inner.fd_manager.limit {
        return Err(ErrorNo::EINVAL as isize);
    }

    let shadow_bitset = ShadowBitset::new(addr, len);
    if addr as usize == 0 {
        return Ok((Vec::new(), Vec::new(), shadow_bitset));
    }

    let start: VirtAddr = (addr as usize).into();
    let end = start + (len + 7) / 8;
    if inner
        .memory_set
        .lock()
        .manual_alloc_range_for_lazy(start, end)
        .is_err()
    {
        return Err(ErrorNo::EFAULT as isize);
    }
    let mut fds = Vec::new();
    let mut files = Vec::new();
    for fd in 0..len {
        if shadow_bitset.check(fd) {
            if let Some(file) = inner.fd_manager.fd_table[fd].as_ref() {
                fds.push(Arc::clone(file));
                files.push(fd);
            } else {
                return Err(ErrorNo::EBADF as isize);
            }
        }
    }

    shadow_bitset.clear();
    Ok((fds, files, shadow_bitset))
}

pub fn syscall_pselect6(
    nfds: usize,
    readfds: *mut usize,
    writefds: *mut usize,
    exceptfds: *mut usize,
    timeout: *const TimeSecs,
    _mask: usize,
) -> isize {
    let (rfiles, rfds, mut rset) = match init_fd_set(readfds, nfds) {
        Ok(ans) => ans,
        Err(e) => return e,
    };
    let (wfiles, wfds, mut wset) = match init_fd_set(writefds, nfds) {
        Ok(ans) => ans,
        Err(e) => return e,
    };
    let (efiles, efds, mut eset) = match init_fd_set(exceptfds, nfds) {
        Ok(ans) => ans,
        Err(e) => return e,
    };
    let process = current_process();
    let inner = process.inner.lock();
    let expire_time = if timeout as usize != 0 {
        if inner
            .memory_set
            .lock()
            .manual_alloc_type_for_lazy(timeout)
            .is_err()
        {
            return ErrorNo::EFAULT as isize;
        }
        riscv::register::time::read() + unsafe { (*timeout).get_ticks() }
    } else {
        usize::MAX
    };
    drop(inner);
    loop {
        let mut set = 0;
        if rset.valid() {
            for i in 0..rfds.len() {
                if rfiles[i].lock().ready_to_read() {
                    rset.set(rfds[i]);
                    set += 1;
                }
            }
        }
        if wset.valid() {
            for i in 0..wfds.len() {
                if wfiles[i].lock().ready_to_write() {
                    wset.set(wfds[i]);
                    set += 1;
                }
            }
        }
        if eset.valid() {
            for i in 0..efds.len() {
                if efiles[i].lock().in_exceptional_conditions() {
                    eset.set(efds[i]);
                    set += 1;
                }
            }
        }
        if set > 0 {
            return set as isize;
        }
        yield_now_task();
        if riscv::register::time::read() > expire_time {
            return 0;
        }
    }
}
