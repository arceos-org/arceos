#[cfg(feature = "fs")]
use syscall_fs::FileDesc;

#[cfg(feature = "fs")]
use syscall_utils::MMAPFlags;

use syscall_utils::{SyscallError, SyscallResult, MMAPPROT};
extern crate alloc;

use axhal::{arch::flush_tlb, mem::VirtAddr, paging::MappingFlags};
use axmem::MemorySet;

use axprocess::current_process;
use bitflags::bitflags;

const MAX_HEAP_SIZE: usize = 0x20000;
/// 修改用户堆大小，
///
/// - 如输入 brk 为 0 ，则返回堆顶地址
/// - 重新设置堆顶地址，如成功则返回设置后的堆顶地址，否则保持不变，并返回之前的堆顶地址。
pub fn syscall_brk(brk: usize) -> SyscallResult {
    let curr_process = current_process();
    let mut return_val: isize = curr_process.get_heap_top() as isize;
    let heap_bottom = curr_process.get_heap_bottom() as usize;
    if brk != 0 {
        if brk >= heap_bottom && brk <= heap_bottom + MAX_HEAP_SIZE {
            curr_process.set_heap_top(brk as u64);
            return_val = brk as isize;
        }
    }
    Ok(return_val)
}

#[cfg(feature = "fs")]
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
) -> SyscallResult {
    use axlog::debug;
    use axmem::MemBackend;

    debug!(
        "mmap start={:x} len={:x} prot=[{:#?}] flags=[{:#?}] fd={} offset={:x}",
        start, len, prot, flags, fd, offset
    );
    let fixed = flags.contains(MMAPFlags::MAP_FIXED);
    // try to map to NULL
    if fixed && start == 0 {
        return Err(SyscallError::EINVAL);
    }

    let process = current_process();

    let addr = if flags.contains(MMAPFlags::MAP_ANONYMOUS) {
        // no file
        if !(fd == -1 && offset == 0) {
            return Err(SyscallError::EINVAL);
        }
        process
            .memory_set
            .lock()
            .mmap(start.into(), len, prot.into(), fixed, None)
    } else {
        // file backend
        debug!("[mmap] fd: {}, offset: 0x{:x}", fd, offset);
        if fd >= process.fd_manager.fd_table.lock().len() as i32 || fd < 0 {
            return Err(SyscallError::EINVAL);
        }
        let file = match &process.fd_manager.fd_table.lock()[fd as usize] {
            // 文件描述符表里面存的是文件描述符，这很合理罢
            Some(file) => alloc::boxed::Box::new(
                file.as_any()
                    .downcast_ref::<FileDesc>()
                    .expect("Try to mmap with a non-file backend")
                    .file
                    .lock()
                    .clone(),
            ),
            // fd not found
            None => return Err(SyscallError::EINVAL),
        };

        let backend = MemBackend::new(file, offset as u64);
        process
            .memory_set
            .lock()
            .mmap(start.into(), len, prot.into(), fixed, Some(backend))
    };

    flush_tlb(None);
    debug!("mmap: 0x{:x}", addr);
    // info!("val: {}", unsafe { *(addr as *const usize) });
    Ok(addr)
}

pub fn syscall_munmap(start: usize, len: usize) -> SyscallResult {
    let process = current_process();
    process.memory_set.lock().munmap(start.into(), len);
    flush_tlb(None);
    Ok(0)
}

pub fn syscall_msync(start: usize, len: usize) -> SyscallResult {
    let process = current_process();
    process.memory_set.lock().msync(start.into(), len);

    Ok(0)
}

pub fn syscall_mprotect(start: usize, len: usize, prot: MMAPPROT) -> SyscallResult {
    let process = current_process();

    process
        .memory_set
        .lock()
        .mprotect(VirtAddr::from(start), len, prot.into());

    flush_tlb(None);
    Ok(0)
}

const IPC_PRIVATE: i32 = 0;

bitflags! {
    #[derive(Debug)]
    struct ShmFlags: i32 {
        const IPC_CREAT = 0o1000;
        const IPC_EXCL = 0o2000;
        // unimplemented:
        const SHM_HUGETLB = 0o4000;
        const SHM_NORESERVE = 0o10000;
    }
}

// TODO: uid and gid support
pub fn syscall_shmget(key: i32, size: usize, flags: i32) -> SyscallResult {
    let pid = current_process().pid();

    // 9 bits for permission
    let mode: u16 = (flags as u16) & ((1 << 10) - 1);

    let Some(flags) = ShmFlags::from_bits(flags - mode as i32) else {
        // return -1;
        return Err(SyscallError::EINVAL);
    };

    if key == IPC_PRIVATE {
        let Ok((shmid, mem)) = MemorySet::create_shared_mem(key, size, pid, 0, 0, mode) else {
            return Err(SyscallError::EINVAL);
        };

        current_process()
            .memory_set
            .lock()
            .add_private_shared_mem(shmid, mem);

        Ok(shmid as isize)
    } else {
        let mut key_map = axmem::KEY_TO_SHMID.lock();

        match key_map.get(&key) {
            Some(shmid) => {
                if flags.contains(ShmFlags::IPC_CREAT) && flags.contains(ShmFlags::IPC_EXCL) {
                    Err(SyscallError::EEXIST)
                } else {
                    Ok(*shmid as isize)
                }
            }
            None => {
                if flags.contains(ShmFlags::IPC_CREAT) {
                    let Ok((shmid, mem)) = MemorySet::create_shared_mem(key, size, pid, 0, 0, mode)
                    else {
                        return Err(SyscallError::EINVAL);
                    };

                    key_map.insert(key, shmid);
                    MemorySet::add_shared_mem(shmid, mem);
                    Ok(shmid as isize)
                } else {
                    Err(SyscallError::ENOENT)
                }
            }
        }
    }
}

bitflags! {
    #[derive(Debug)]
    struct ShmAtFlags: i32 {
        const SHM_RND = 0o20000;
        const SHM_EXEC = 0o100000;
        const SHM_RDONLY = 0o10000;
        const SHM_REMAP = 0o40000;
    }
}

pub fn syscall_shmat(shmid: i32, addr: usize, flags: i32) -> SyscallResult {
    let process = current_process();

    let mut memory = process.memory_set.lock();

    let flags = ShmAtFlags::from_bits(flags).unwrap();

    let Some(mem) = memory
        .get_private_shared_mem(shmid)
        .or_else(|| MemorySet::get_shared_mem(shmid))
    else {
        return Err(SyscallError::EINVAL);
    };
    let size = mem.size();

    let addr = if addr == 0 {
        match memory.find_free_area(addr.into(), size) {
            Some(addr) => addr,
            None => return Err(SyscallError::ENOMEM),
        }
    } else {
        let addr: VirtAddr = addr.into();
        let addr = if addr.is_aligned_4k() {
            addr
        } else {
            if flags.contains(ShmAtFlags::SHM_RND) {
                addr.align_up_4k()
            } else {
                return Err(SyscallError::EINVAL);
            }
        };

        if flags.contains(ShmAtFlags::SHM_REMAP) {
            memory.split_for_area(addr, size);
            flush_tlb(None);
        } else {
            unimplemented!()
        }

        addr
    };

    let mut map_flags = MappingFlags::USER;
    if flags.contains(ShmAtFlags::SHM_RDONLY) {
        map_flags |= MappingFlags::READ;
    } else {
        map_flags |= MappingFlags::READ | MappingFlags::WRITE;
    }
    if flags.contains(ShmAtFlags::SHM_EXEC) {
        map_flags |= MappingFlags::EXECUTE;
    }

    memory.attach_shared_mem(mem, addr, map_flags);
    flush_tlb(None);

    Ok(addr.as_usize() as isize)
}
