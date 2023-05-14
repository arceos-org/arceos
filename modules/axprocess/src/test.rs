use alloc::sync::Arc;
use axhal::arch::write_page_table_root;
use axlog::info;
use axmem::paging::KERNEL_PAGE_TABLE;
use axtask::{TaskId, RUN_QUEUE};
use spinlock::SpinNoIrq;
extern crate alloc;
use crate::process::{yield_now_task, Process, KERNEL_PROCESS_ID, PID2PC};
use alloc::boxed::Box;
use alloc::string::{self, String, ToString};
use alloc::vec::Vec;
use riscv::asm;
/// 该文件用于进行运行测例

const TESTCASES: &[&str] = &[
    "brk",
    "chdir",
    "clone",
    "close",
    "dup",
    "dup2",
    "execve",
    "exit",
    "fork",
    "fstat",
    "getcwd",
    "getdents",
    "getpid",
    "getppid",
    "gettimeofday",
    "mkdir_",
    "mmap",
    "mount",
    "munmap",
    "open",
    "openat",
    "pipe",
    "read",
    "sleep",
    "times",
    "umount",
    "uname",
    "unlink",
    "wait",
    "waitpid",
    "write",
    "yield",
];

/// 运行测试时的状态机，记录测试结果与内容
struct TestResult {
    sum: usize,
    accepted: usize,
    now_testcase: Option<(String, usize)>, // 同时记录名称与进程号
    failed_testcases: Vec<String>,
}

impl TestResult {
    pub fn new(case_num: usize) -> Self {
        Self {
            sum: case_num,
            accepted: 0,
            now_testcase: None,
            failed_testcases: Vec::new(),
        }
    }
    pub fn load(&mut self, testcase: &String, pid: usize) {
        info!(
            " --------------- load testcase: {} --------------- ",
            testcase
        );
        self.now_testcase = Some((testcase.into(), pid));
    }
    /// 调用这个函数的应当是测例最开始的进程，而不是它fork出来的一系列进程
    /// 认为exit_code为负数时代表不正常
    pub fn finish_one_test(&mut self, exit_code: i32, pid: usize) {
        // 每一个进程都会调用当前函数，所以要检查是否是主进程退出，即测例完全结束
        // 一开始记录了主进程的进程号，只有对应进程号的进程退出时才认为是测例结束
        if let Some((_, main_pid)) = self.now_testcase {
            if main_pid != pid {
                return;
            }
        } else {
            panic!("Error when finish one test: no testcase loaded");
        }
        match exit_code {
            0 => {
                info!(" --------------- test passed --------------- ");
                self.accepted += 1;
                self.now_testcase.take();
            }
            _ => {
                info!(
                    " --------------- TEST FAILED, exit code = {} --------------- ",
                    exit_code
                );
                self.failed_testcases
                    .push(self.now_testcase.take().unwrap().0);
            }
        }
    }

    /// 完成了所有测例之后，打印测试结果
    pub fn show_result(&self) {
        info!(
            " --------------- all test ended, passed {} / {} --------------- ",
            self.accepted, self.sum
        );
        info!(" --------------- failed tests: --------------- ");
        for test in &self.failed_testcases {
            info!("{}", test);
        }
        info!(" --------------- end --------------- ");
    }
}
lazy_static::lazy_static! {
    static ref TESTITER: SpinNoIrq<Box<dyn Iterator<Item = &'static &'static str> + Send>> = SpinNoIrq::new(Box::new(TESTCASES.iter()));
    static ref TESTRESULT: SpinNoIrq<TestResult> = SpinNoIrq::new(TestResult::new(TESTCASES.len()));
}

/// 某一个测试用例完成之后调用，记录测试结果
pub fn finish_one_test(exit_code: i32, pid: usize) {
    TESTRESULT.lock().finish_one_test(exit_code, pid);
}

pub fn show_result() {
    TESTRESULT.lock().show_result();
}

/// 执行运行所有测例的任务
pub fn run_testcases() {
    loop {
        let ans = TESTITER.lock().next().map_or_else(
            || {
                // 已经执行完所有测例，输出测试结果并且跳出
                TESTRESULT.lock().show_result();
                None
            },
            |&testcase| {
                // 清空分配器
                TaskId::clear();
                unsafe {
                    write_page_table_root(KERNEL_PAGE_TABLE.root_paddr());
                    asm::sfence_vma_all();
                };
                let main_task = Process::new(testcase);
                let now_process_id = main_task.get_process_id();

                TESTRESULT
                    .lock()
                    .load(&(testcase.to_string()), now_process_id as usize);
                RUN_QUEUE.lock().add_task(main_task);
                loop {
                    if PID2PC.lock().get(&now_process_id).is_none() {
                        // 若已经找不到对应的进程，说明已经被释放
                        // 他的task_status已经记录在了TASKRESULT中
                        // 将自己从父进程的孩子序列中移去
                        break Some(now_process_id);
                    }
                    yield_now_task();
                }
            },
        );
        if let Some(process_id) = ans {
            let kernel_process = Arc::clone(PID2PC.lock().get(&KERNEL_PROCESS_ID).unwrap());
            kernel_process
                .inner
                .lock()
                .children
                .retain(|x| x.pid != process_id);
            // 去除指针引用，此时process_id对应的进程已经被释放
        } else {
            // 已经测试完所有的测例
            break;
        }
    }
    panic!("All test finish!");
}
