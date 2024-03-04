use core::ops::Deref;
use core::ptr::copy_nonoverlapping;
use core::str::from_utf8;
extern crate alloc;
use alloc::sync::Arc;
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use axconfig::{MAX_USER_HEAP_SIZE, MAX_USER_STACK_SIZE, USER_HEAP_BASE, USER_STACK_TOP};
use axerrno::{AxError, AxResult};
use axhal::arch::flush_tlb;
use axhal::mem::VirtAddr;
use axhal::paging::MappingFlags;
use axhal::KERNEL_PROCESS_ID;
use axlog::{debug, info};
use axmem::MemorySet;
#[cfg(feature = "signal")]
use axsignal::signal_no::SignalNo;
use axsync::Mutex;
use axtask::{current, yield_now, CurrentTask, TaskId, TaskState, IDLE_TASK, RUN_QUEUE};
use elf_parser::{
    get_app_stack_region, get_auxv_vector, get_elf_entry, get_elf_segments, get_relocate_pairs,
};
use xmas_elf::program::SegmentData;

use crate::flags::WaitStatus;
use crate::futex::clear_wait;
use crate::link::real_path;
use crate::process::{Process, PID2PC, TID2TASK};
#[cfg(feature = "signal")]
use crate::signal::{send_signal_to_process, send_signal_to_thread};

/// 初始化内核调度进程
pub fn init_kernel_process() {
    let kernel_process = Arc::new(Process::new(
        TaskId::new().as_u64(),
        0,
        Arc::new(Mutex::new(MemorySet::new_empty())),
        0,
        vec![],
    ));

    axtask::init_scheduler();
    kernel_process.tasks.lock().push(Arc::clone(unsafe {
        IDLE_TASK.current_ref_raw().get_unchecked()
    }));
    PID2PC.lock().insert(kernel_process.pid(), kernel_process);
}

pub fn current_process() -> Arc<Process> {
    let current_task = current();

    let current_process = Arc::clone(PID2PC.lock().get(&current_task.get_process_id()).unwrap());

    current_process
}

/// 退出当前任务
pub fn exit_current_task(exit_code: i32) -> ! {
    let process = current_process();
    let current_task = current();

    let curr_id = current_task.id().as_u64();

    info!("exit task id {} with code _{}_", curr_id, exit_code);
    clear_wait(
        if current_task.is_leader() {
            process.pid()
        } else {
            curr_id
        },
        current_task.is_leader(),
    );
    // 检查这个任务是否有sig_child信号
    #[cfg(feature = "signal")]
    if current_task.get_sig_child() || current_task.is_leader() {
        let parent = process.get_parent();
        if parent != KERNEL_PROCESS_ID {
            // 发送sigchild
            send_signal_to_process(parent as isize, 17).unwrap();
        }
    }
    // clear_child_tid 的值不为 0，则将这个用户地址处的值写为0
    let clear_child_tid = current_task.get_clear_child_tid();
    if clear_child_tid != 0 {
        // 先确认是否在用户空间
        if process
            .manual_alloc_for_lazy(clear_child_tid.into())
            .is_ok()
        {
            unsafe {
                *(clear_child_tid as *mut i32) = 0;
            }
        }
    }
    if current_task.is_leader() {
        loop {
            let mut all_exited = true;

            for task in process.tasks.lock().deref() {
                if !task.is_leader() && task.state() != TaskState::Exited {
                    all_exited = false;
                    break;
                }
            }
            if !all_exited {
                yield_now();
            } else {
                break;
            }
        }
        TID2TASK.lock().remove(&curr_id);
        process.set_exit_code(exit_code);

        process.set_zombie(true);

        process.tasks.lock().clear();
        process.fd_manager.fd_table.lock().clear();
        #[cfg(feature = "signal")]
        process.signal_modules.lock().clear();

        let mut pid2pc = PID2PC.lock();
        let kernel_process = pid2pc.get(&KERNEL_PROCESS_ID).unwrap();
        // 将子进程交给idle进程
        // process.memory_set = Arc::clone(&kernel_process.memory_set);
        for child in process.children.lock().deref() {
            child.set_parent(KERNEL_PROCESS_ID);
            kernel_process.children.lock().push(Arc::clone(child));
        }
        if let Some(parent_process) = pid2pc.get(&process.get_parent()) {
            parent_process.set_vfork_block(false);
        }
        pid2pc.remove(&process.pid());
        drop(pid2pc);
        drop(process);
    } else {
        TID2TASK.lock().remove(&curr_id);
        // 从进程中删除当前线程
        let mut tasks = process.tasks.lock();
        let len = tasks.len();
        for index in 0..len {
            if tasks[index].id().as_u64() == curr_id {
                tasks.remove(index);
                break;
            }
        }
        drop(tasks);
        #[cfg(feature = "signal")]
        process.signal_modules.lock().remove(&curr_id);
        drop(process);
    }
    RUN_QUEUE.lock().exit_current(exit_code);
}

/// 返回应用程序入口，用户栈底，用户堆底
pub fn load_app(
    name: String,
    mut args: Vec<String>,
    envs: &Vec<String>,
    memory_set: &mut MemorySet,
) -> AxResult<(VirtAddr, VirtAddr, VirtAddr)> {
    if name.ends_with(".sh") {
        args = [vec![String::from("busybox"), String::from("sh")], args].concat();
        return load_app("busybox".to_string(), args, envs, memory_set);
    }
    let elf_data = if let Ok(ans) = axfs::api::read(name.as_str()) {
        ans
    } else {
        // exit(0)
        return Err(AxError::NotFound);
    };
    let elf = xmas_elf::ElfFile::new(&elf_data).expect("Error parsing app ELF file.");
    debug!("app elf data length: {}", elf_data.len());
    if let Some(interp) = elf
        .program_iter()
        .find(|ph| ph.get_type() == Ok(xmas_elf::program::Type::Interp))
    {
        let interp = match interp.get_data(&elf) {
            Ok(SegmentData::Undefined(data)) => data,
            _ => panic!("Invalid data in Interp Elf Program Header"),
        };

        let interp_path = from_utf8(interp).expect("Interpreter path isn't valid UTF-8");
        // remove trailing '\0'
        let interp_path = interp_path.trim_matches(char::from(0)).to_string();
        let real_interp_path = real_path(&interp_path);
        args = [vec![real_interp_path.clone()], args].concat();
        return load_app(real_interp_path, args, envs, memory_set);
    }
    info!("args: {:?}", args);
    let elf_base_addr = Some(0x400_0000);
    axlog::warn!("The elf base addr may be different in different arch!");
    // let (entry, segments, relocate_pairs) = parse_elf(&elf, elf_base_addr);
    let entry = get_elf_entry(&elf, elf_base_addr);
    let segments = get_elf_segments(&elf, elf_base_addr);
    let relocate_pairs = get_relocate_pairs(&elf, elf_base_addr);
    for segment in segments {
        memory_set.new_region(
            segment.vaddr,
            segment.size,
            segment.flags,
            segment.data.as_deref(),
            None,
        );
    }

    for relocate_pair in relocate_pairs {
        let src: usize = relocate_pair.src.into();
        let dst: usize = relocate_pair.dst.into();
        let count = relocate_pair.count;
        unsafe { copy_nonoverlapping(src.to_ne_bytes().as_ptr(), dst as *mut u8, count) }
    }

    // Now map the stack and the heap
    let heap_start = VirtAddr::from(USER_HEAP_BASE);
    let heap_data = [0_u8].repeat(MAX_USER_HEAP_SIZE);
    memory_set.new_region(
        heap_start,
        MAX_USER_HEAP_SIZE,
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
        Some(&heap_data),
        None,
    );
    info!(
        "[new region] user heap: [{:?}, {:?})",
        heap_start,
        heap_start + MAX_USER_HEAP_SIZE
    );

    let auxv = get_auxv_vector(&elf, elf_base_addr);

    let stack_top = VirtAddr::from(USER_STACK_TOP);
    let stack_size = MAX_USER_STACK_SIZE;

    let (stack_data, stack_bottom) = get_app_stack_region(args, envs, auxv, stack_top, stack_size);
    memory_set.new_region(
        stack_top,
        stack_size,
        MappingFlags::USER | MappingFlags::READ | MappingFlags::WRITE,
        Some(&stack_data),
        None,
    );
    info!(
        "[new region] user stack: [{:?}, {:?})",
        stack_top,
        stack_top + stack_size
    );
    Ok((entry, stack_bottom.into(), heap_start))
}

/// 当从内核态到用户态时，统计对应进程的时间信息
pub fn time_stat_from_kernel_to_user() {
    let curr_task = current();
    curr_task.time_stat_from_kernel_to_user();
}

#[no_mangle]
/// 当从用户态到内核态时，统计对应进程的时间信息
pub fn time_stat_from_user_to_kernel() {
    let curr_task = current();
    curr_task.time_stat_from_user_to_kernel();
}

/// 统计时间输出
/// (用户态秒，用户态微妙，内核态秒，内核态微妙)
pub fn time_stat_output() -> (usize, usize, usize, usize) {
    let curr_task = current();
    curr_task.time_stat_output()
}

pub fn handle_page_fault(addr: VirtAddr, flags: MappingFlags) {
    axlog::debug!("'page fault' addr: {:?}, flags: {:?}", addr, flags);
    let current_process = current_process();
    axlog::debug!(
        "memory token : {:#x}",
        current_process.memory_set.lock().page_table_token()
    );

    if current_process
        .memory_set
        .lock()
        .handle_page_fault(addr, flags)
        .is_ok()
    {
        // Change flush all memory to just the error page addr.
        flush_tlb(Some(addr));
    } else {
        #[cfg(feature = "signal")]
        let _ = send_signal_to_thread(current().id().as_u64() as isize, SignalNo::SIGSEGV as isize);
    }
}

/// 在当前进程找对应的子进程，并等待子进程结束
/// 若找到了则返回对应的pid
/// 否则返回一个状态
///
/// # Safety
///
/// 保证传入的 ptr 是有效的
pub unsafe fn wait_pid(pid: isize, exit_code_ptr: *mut i32) -> Result<u64, WaitStatus> {
    // 获取当前进程
    let curr_process = current_process();
    let mut exit_task_id: usize = 0;
    let mut answer_id: u64 = 0;
    let mut answer_status = WaitStatus::NotExist;
    for (index, child) in curr_process.children.lock().iter().enumerate() {
        if pid == -1 {
            // 任意一个进程结束都可以的
            answer_status = WaitStatus::Running;
            if let Some(exit_code) = child.get_code_if_exit() {
                answer_status = WaitStatus::Exited;
                info!("wait pid _{}_ with code _{}_", child.pid(), exit_code);
                exit_task_id = index;
                if !exit_code_ptr.is_null() {
                    unsafe {
                        // 因为没有切换页表，所以可以直接填写
                        *exit_code_ptr = exit_code << 8;
                    }
                }
                answer_id = child.pid();
                break;
            }
        } else if child.pid() == pid as u64 {
            // 找到了对应的进程
            if let Some(exit_code) = child.get_code_if_exit() {
                answer_status = WaitStatus::Exited;
                info!("wait pid _{}_ with code _{:?}_", child.pid(), exit_code);
                exit_task_id = index;
                if !exit_code_ptr.is_null() {
                    unsafe {
                        *exit_code_ptr = exit_code << 8;
                        // 用于WEXITSTATUS设置编码
                    }
                }
                answer_id = child.pid();
            } else {
                answer_status = WaitStatus::Running;
            }
            break;
        }
    }
    // 若进程成功结束，需要将其从父进程的children中删除
    if answer_status == WaitStatus::Exited {
        curr_process.children.lock().remove(exit_task_id);
        return Ok(answer_id);
    }
    Err(answer_status)
}

/// 以进程作为中转调用task的yield
pub fn yield_now_task() {
    axtask::yield_now();
}

pub fn sleep_now_task(dur: core::time::Duration) {
    axtask::sleep(dur);
}

pub fn current_task() -> CurrentTask {
    axtask::current()
}

pub fn set_child_tid(tid: usize) {
    let curr = current_task();
    curr.set_clear_child_tid(tid);
}
