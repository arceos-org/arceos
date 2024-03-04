use axfs::api::FileIO;
use axhal::{mem::VirtAddr, time::current_ticks};
use axprocess::{current_process, yield_now_task};
use bitflags::bitflags;
extern crate alloc;
use alloc::{sync::Arc, vec::Vec};
use syscall_utils::{SyscallError, SyscallResult, TimeSecs, TimeVal};
bitflags! {
    /// 在文件上等待或者发生过的事件
    #[derive(Clone, Copy,Debug)]
    pub struct PollEvents: u16 {
        /// 可读
        const IN = 0x0001;
        /// 可写
        const OUT = 0x0004;
        /// 错误
        const ERR = 0x0008;
        /// 挂起,如pipe另一端关闭
        const HUP = 0x0010;
        /// 无效的事件
        const NVAL = 0x0020;
    }
}

#[derive(Default)]
struct PpollFdSet {
    files: Vec<Arc<dyn FileIO>>,
    fds: Vec<usize>,
    shadow_bitset: ShadowBitset,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PollFd {
    /// 等待的fd
    pub fd: i32,
    /// 等待的事件
    pub events: PollEvents,
    /// 返回的事件
    pub revents: PollEvents,
}

// 定义一个bitset,用于查找掩码
struct ShadowBitset {
    addr: *mut usize,
    len: usize, // 是包含的bit数目,而不是字节数目
}

impl Default for ShadowBitset {
    fn default() -> Self {
        Self {
            addr: core::ptr::null_mut(),
            len: 0,
        }
    }
}

impl ShadowBitset {
    pub fn new(addr: *mut usize, len: usize) -> Self {
        Self { addr, len }
    }

    pub fn check(&self, index: usize) -> bool {
        if index >= self.len {
            return false;
        }
        // 因为一次add会移动八个字节,所以这里需要除以64,即8个字节,每一个字节8位
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

/// 实现ppoll系统调用
///
/// fds：一个PollFd列表
/// expire_time：时间戳,用来记录是否超时
///
/// 返回值：(usize, Vec<PollFd>) 第一个参数遵守 ppoll 系统调用的返回值约定,第二个参数为返回的 `PollFd` 列表
fn ppoll(mut fds: Vec<PollFd>, expire_time: usize) -> (isize, Vec<PollFd>) {
    loop {
        // 满足事件要求而被触发的事件描述符数量
        let mut set: isize = 0;
        let process = current_process();
        for poll_fd in &mut fds {
            let fd_table = process.fd_manager.fd_table.lock();
            if let Some(file) = fd_table[poll_fd.fd as usize].as_ref() {
                poll_fd.revents = PollEvents::empty();
                // let file = file.lock();
                if file.in_exceptional_conditions() {
                    poll_fd.revents |= PollEvents::ERR;
                }
                if file.is_hang_up() {
                    poll_fd.revents |= PollEvents::HUP;
                }
                if poll_fd.events.contains(PollEvents::IN) && file.ready_to_read() {
                    poll_fd.revents |= PollEvents::IN;
                }
                if poll_fd.events.contains(PollEvents::OUT) && file.ready_to_write() {
                    poll_fd.revents |= PollEvents::OUT;
                }
                // 如果返回事件不为空,代表有响应
                if !poll_fd.revents.is_empty() {
                    set += 1;
                }
            } else {
                // 不存在也是一种响应
                poll_fd.revents = PollEvents::ERR;
                set += 1;
            }
        }
        if set > 0 {
            return (set, fds);
        }
        if current_ticks() as usize > expire_time {
            // 过期了,直接返回
            return (0, fds);
        }
        yield_now_task();

        #[cfg(feature = "signal")]
        if process.have_signals().is_some() {
            // 有信号,此时停止处理,直接返回
            return (0, fds);
        }
    }
}

/// 实现ppoll系统调用
///
/// 其中timeout是一段相对时间,需要计算出相对于当前时间戳的绝对时间戳
///
/// # Arguments
/// * `ufds` - *mut PollFd
/// * `nfds` - usize
/// * `timeout` - *const TimeSecs
/// * `mask` - usize
pub fn syscall_ppoll(args: [usize; 6]) -> SyscallResult {
    let ufds = args[0] as *mut PollFd;
    let nfds = args[1];
    let timeout = args[2] as *const TimeSecs;
    let _mask = args[3];
    let process = current_process();

    let start: VirtAddr = (ufds as usize).into();
    let end = start + nfds * core::mem::size_of::<PollFd>();
    if process.manual_alloc_range_for_lazy(start, end).is_err() {
        return Err(SyscallError::EFAULT);
    }

    let mut fds: Vec<PollFd> = Vec::new();

    for i in 0..nfds {
        unsafe {
            fds.push(*(ufds.add(i)));
        }
    }

    let expire_time = if timeout as usize != 0 {
        if process.manual_alloc_type_for_lazy(timeout).is_err() {
            return Err(SyscallError::EFAULT);
        }
        current_ticks() as usize + unsafe { (*timeout).get_ticks() }
    } else {
        usize::MAX
    };

    let (set, ret_fds) = ppoll(fds, expire_time);
    // 将得到的fd存储到原先的指针中
    for (i, fd) in ret_fds.iter().enumerate() {
        unsafe {
            *(ufds.add(i)) = *fd;
        }
    }
    Ok(set)
}

/// 实现poll系统调用
/// # Arguments
/// * `ufds` - *mut PollFd
/// * `nfds` - usize
/// * `timeout_msecs` - usize
pub fn syscall_poll(args: [usize; 6]) -> SyscallResult {
    let ufds = args[0] as *mut PollFd;
    let nfds = args[1];
    let timeout_msecs = args[2];
    let process = current_process();

    let start: VirtAddr = (ufds as usize).into();
    let end = start + nfds * core::mem::size_of::<PollFd>();
    if process.manual_alloc_range_for_lazy(start, end).is_err() {
        return Err(SyscallError::EFAULT);
    }

    let mut fds: Vec<PollFd> = Vec::new();

    for i in 0..nfds {
        unsafe {
            fds.push(*(ufds.add(i)));
        }
    }
    let expire_time =
        current_ticks() as usize + TimeVal::from_micro(timeout_msecs).to_ticks() as usize;

    let (set, ret_fds) = ppoll(fds, expire_time);
    // 将得到的fd存储到原先的指针中
    for (i, fd) in ret_fds.iter().enumerate() {
        unsafe {
            *(ufds.add(i)) = *fd;
        }
    }
    Ok(set)
}

/// 根据给定的地址和长度新建一个fd set,包括文件描述符指针数组,文件描述符数值数组,以及一个bitset
fn init_fd_set(addr: *mut usize, len: usize) -> Result<PpollFdSet, SyscallError> {
    let process = current_process();
    if len >= process.fd_manager.get_limit() as usize {
        axlog::error!(
            "[pselect6()] len {len} >= limit {}",
            process.fd_manager.get_limit()
        );
        return Err(SyscallError::EINVAL);
    }

    let shadow_bitset = ShadowBitset::new(addr, len);
    if addr.is_null() {
        return Ok(PpollFdSet {
            shadow_bitset,
            ..Default::default()
        });
    }

    let start: VirtAddr = (addr as usize).into();
    let end = start + (len + 7) / 8;
    if process.manual_alloc_range_for_lazy(start, end).is_err() {
        axlog::error!("[pselect6()] addr {addr:?} invalid");
        return Err(SyscallError::EFAULT);
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
                return Err(SyscallError::EBADF);
            }
        }
    }

    shadow_bitset.clear();

    Ok(PpollFdSet {
        files,
        fds,
        shadow_bitset,
    })
}

// 实现 select 系统调用
/// # Arguments
/// * `nfds` - usize
/// * `readfds` - *mut usize
/// * `writefds` - *mut usize
/// * `exceptfds` - *mut usize
/// * `timeout` - *const TimeSecs
pub fn syscall_select(mut args: [usize; 6]) -> SyscallResult {
    args[5] = 0;
    syscall_pselect6(args)
}
/// 实现pselect6系统调用
/// # Arguments
/// * `nfds` - usize
/// * `readfds` - *mut usize
/// * `writefds` - *mut usize
/// * `exceptfds` - *mut usize
/// * `timeout` - *const TimeSecs
/// * `mask` - usize
pub fn syscall_pselect6(args: [usize; 6]) -> SyscallResult {
    let nfds = args[0];
    let readfds = args[1] as *mut usize;
    let writefds = args[2] as *mut usize;
    let exceptfds = args[3] as *mut usize;
    let timeout = args[4] as *const TimeSecs;
    let _mask = args[5];
    let (rfiles, rfds, mut rset) = match init_fd_set(readfds, nfds) {
        Ok(ans) => (ans.files, ans.fds, ans.shadow_bitset),
        Err(e) => return Err(e),
    };
    let (wfiles, wfds, mut wset) = match init_fd_set(writefds, nfds) {
        Ok(ans) => (ans.files, ans.fds, ans.shadow_bitset),
        Err(e) => return Err(e),
    };
    let (efiles, efds, mut eset) = match init_fd_set(exceptfds, nfds) {
        Ok(ans) => (ans.files, ans.fds, ans.shadow_bitset),
        Err(e) => return Err(e),
    };
    let process = current_process();

    let expire_time = if !timeout.is_null() {
        if process
            .memory_set
            .lock()
            .manual_alloc_type_for_lazy(timeout)
            .is_err()
        {
            axlog::error!("[pselect6()] timeout addr {timeout:?} invalid");
            return Err(SyscallError::EFAULT);
        }
        current_ticks() as usize + unsafe { (*timeout).get_ticks() }
    } else {
        usize::MAX
    };

    axlog::debug!("[pselect6()]: r: {rfds:?}, w: {wfds:?}, e: {efds:?}");

    loop {
        // Why yield first?
        //
        // 当用户程序中出现如下结构：
        // while (true) { select(); }
        // 如果存在 ready 的 fd,select() 立即返回,
        // 但并不完全满足用户程序的要求,可能出现死循环。
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
            return Ok(set as isize);
        }
        if current_ticks() as usize > expire_time {
            return Ok(0);
        }
        #[cfg(feature = "signal")]
        if process.have_signals().is_some() {
            return Err(SyscallError::EINTR);
        }
    }
}
