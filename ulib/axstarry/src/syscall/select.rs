extern crate alloc;
use alloc::{sync::Arc, vec::Vec};
use axfs::api::FileIO;
use axhal::{mem::VirtAddr, time::current_ticks};
use axlog::{debug, error};
use axprocess::{current_process, yield_now_task};

use super::{flags::TimeSecs, ErrorNo};

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
        let byte_index = index / 64;
        let bit_index = index & 0x3f;
        unsafe { *self.addr.add(byte_index) & (1 << bit_index) != 0 }
    }

    pub fn set(&mut self, index: usize) {
        if index >= self.len {
            return;
        }
        let byte_index = index / 64;
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
) -> Result<(Vec<Arc<dyn FileIO>>, Vec<usize>, ShadowBitset), isize> {
    let process = current_process();
    if len >= process.fd_manager.get_limit() as usize {
        error!(
            "[pselect6()] len {len} >= limit {}",
            process.fd_manager.get_limit()
        );
        return Err(ErrorNo::EINVAL as isize);
    }

    let shadow_bitset = ShadowBitset::new(addr, len);
    if addr.is_null() {
        return Ok((Vec::new(), Vec::new(), shadow_bitset));
    }

    let start: VirtAddr = (addr as usize).into();
    let end = start + (len + 7) / 8;
    if process.manual_alloc_range_for_lazy(start, end).is_err() {
        error!("[pselect6()] addr {addr:?} invalid");
        return Err(ErrorNo::EFAULT as isize);
    }

    let mut fds = Vec::new();
    let mut files = Vec::new();
    for fd in 0..len {
        if shadow_bitset.check(fd) {
            let fd_table = process.fd_manager.fd_table.lock();
            if let Some(file) = fd_table[fd].as_ref() {
                files.push(Arc::clone(file));
                fds.push(fd);
            } else {
                return Err(ErrorNo::EBADF as isize);
            }
        }
    }

    shadow_bitset.clear();
    Ok((files, fds, shadow_bitset))
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

    let expire_time = if !timeout.is_null() {
        if process
            .memory_set
            .lock()
            .manual_alloc_type_for_lazy(timeout)
            .is_err()
        {
            error!("[pselect6()] timeout addr {timeout:?} invalid");
            return ErrorNo::EFAULT as isize;
        }
        current_ticks() as usize + unsafe { (*timeout).get_ticks() }
    } else {
        usize::MAX
    };

    debug!("[pselect6()]: r: {rfds:?}, w: {wfds:?}, e: {efds:?}");

    loop {
        // Why yield first?
        //
        // 当用户程序中出现如下结构：
        // while (true) { select(); }
        // 如果存在 ready 的 fd，select() 立即返回，
        // 但并不完全满足用户程序的要求，可能出现死循环。
        //
        // 因此先 yield 避免其他进程 starvation。
        //
        // 可见 iperf 测例。
        yield_now_task();

        let mut set = 0;
        if rset.valid() {
            for i in 0..rfds.len() {
                if rfiles[i].ready_to_read() {
                    rset.set(rfds[i]);
                    set += 1;
                }
            }
        }
        if wset.valid() {
            for i in 0..wfds.len() {
                if wfiles[i].ready_to_write() {
                    wset.set(wfds[i]);
                    set += 1;
                }
            }
        }
        if eset.valid() {
            for i in 0..efds.len() {
                if efiles[i].in_exceptional_conditions() {
                    eset.set(efds[i]);
                    set += 1;
                }
            }
        }
        if set > 0 {
            return set as isize;
        }
        if current_ticks() as usize > expire_time {
            return 0;
        }

        if process.have_signals().is_some() {
            return ErrorNo::EINTR as isize;
        }
    }
}
