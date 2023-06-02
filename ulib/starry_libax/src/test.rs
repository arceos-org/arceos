use alloc::sync::Arc;
use axerrno::AxResult;
use axfs::api;
use axhal::arch::write_page_table_root;
use axlog::{debug, info};
use axruntime::KERNEL_PAGE_TABLE;
use axtask::{
    macro_task::{EXITED_TASKS, RUN_QUEUE},
    TaskId,
};
use spinlock::SpinNoIrq;

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use axprocess::process::{wait_pid, yield_now_task, Process, KERNEL_PROCESS_ID, PID2PC};
use riscv::asm;

/// 初赛测例
#[allow(dead_code)]
const JUNIOR_TESTCASES: &[&str] = &[
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

/// libc静态测例
pub const LIBC_STATIC_TESTCASES: &[&str] = &[
    "argv",
    "basename",
    "clocale_mbfuncs",
    "clock_gettime",
    "crypt",
    "daemon_failure",
    "dirname",
    "dn_expand_empty",
    "dn_expand_ptr_0",
    "env",
    "fdopen",
    "fflush_exit",
    "fgets_eof",
    "fgetwc_buffering",
    "fnmatch",
    "fpclassify_invalid_ld80",
    "fscanf",
    "ftello_unflushed_append",
    "fwscanf",
    "getpwnam_r_crash",
    "getpwnam_r_errno",
    "iconv_open",
    "iconv_roundtrips",
    "inet_ntop_v4mapped",
    "inet_pton",
    "inet_pton_empty_last_field",
    "iswspace_null",
    "lrand48_signextend",
    "lseek_large",
    "malloc_0",
    "mbc",
    "mbsrtowcs_overflow",
    "memmem_oob",
    "memmem_oob_read",
    "memstream",
    "mkdtemp_failure",
    "mkstemp_failure",
    "pleval",
    "printf_1e9_oob",
    "printf_fmt_g_round",
    "printf_fmt_g_zeros",
    "printf_fmt_n",
    "pthread_cancel",
    "pthread_cancel_points",
    "pthread_cancel_sem_wait",
    "pthread_cond",
    "pthread_cond_smasher",
    "pthread_condattr_setclock",
    "pthread_exit_cancel",
    "pthread_once_deadlock",
    "pthread_robust_detach",
    "pthread_rwlock_ebusy",
    "pthread_tsd",
    "putenv_doublefree",
    "qsort",
    "random",
    "regex_backref_0",
    "regex_bracket_icase",
    "regex_ere_backref",
    "regex_escaped_high_byte",
    "regex_negated_range",
    "regexec_nosub",
    "rewind_clear_error",
    "rlimit_open_files",
    "scanf_bytes_consumed",
    "scanf_match_literal_eof",
    "scanf_nullbyte_char",
    "search_hsearch",
    "search_insque",
    "search_lsearch",
    "search_tsearch",
    "setjmp",
    "setvbuf_unget",
    "sigprocmask_internal",
    "snprintf",
    "socket",
    "sscanf",
    "sscanf_eof",
    //"sscanf_long",
    "stat",
    "statvfs",
    "strftime",
    "string",
    "string_memcpy",
    "string_memmem",
    "string_memset",
    "string_strchr",
    "string_strcspn",
    "string_strstr",
    "strptime",
    "strtod",
    "strtod_simple",
    "strtof",
    "strtol",
    "strtold",
    "strverscmp",
    "swprintf",
    "syscall_sign_extend",
    "tgmath",
    "time",
    "udiv",
    "ungetc",
    "uselocale_0",
    "utime",
    "wcsncpy_read_overflow",
    "wcsstr",
    "wcsstr_false_negative",
    "wcstol",
];

/// 运行测试时的状态机，记录测试结果与内容
struct TestResult {
    sum: usize,
    accepted: usize,
    now_testcase: Option<String>,
    // 同时记录名称与进程号
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
    pub fn load(&mut self, testcase: &String) {
        info!(
            " --------------- load testcase: {} --------------- ",
            testcase
        );
        self.now_testcase = Some(testcase.clone());
    }
    /// 调用这个函数的应当是测例最开始的进程，而不是它fork出来的一系列进程
    /// 认为exit_code为负数时代表不正常
    pub fn finish_one_test(&mut self, exit_code: i32) {
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
                    .push(self.now_testcase.take().unwrap());
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
    static ref TESTITER: SpinNoIrq<Box<dyn Iterator<Item = &'static &'static str> + Send>> = SpinNoIrq::new(Box::new(JUNIOR_TESTCASES.iter()));
    static ref TESTRESULT: SpinNoIrq<TestResult> = SpinNoIrq::new(TestResult::new(JUNIOR_TESTCASES.len()));
}

/// 某一个测试用例完成之后调用，记录测试结果
pub fn finish_one_test(exit_code: i32) {
    TESTRESULT.lock().finish_one_test(exit_code);
}

#[allow(dead_code)]
pub fn show_result() {
    TESTRESULT.lock().show_result();
}
/// 分割命令行参数
fn get_args(command_line: &[u8]) -> Vec<String> {
    let mut args = Vec::new();
    // 需要判断是否存在引号，如busybox_cmd.txt的第一条echo指令便有引号
    // 若有引号时，不能把引号加进去，同时要注意引号内的空格不算是分割的标志
    let mut in_quote = false;
    let mut arg_start = 0; // 一个新的参数的开始位置
    for pos in 0..command_line.len() {
        if command_line[pos] == '\"' as u8 {
            in_quote = !in_quote;
        }
        if command_line[pos] == ' ' as u8 && !in_quote {
            // 代表要进行分割
            // 首先要防止是否有空串
            if arg_start != pos {
                args.push(
                    core::str::from_utf8(&command_line[arg_start..pos])
                        .unwrap()
                        .to_string(),
                );
            }
            arg_start = pos + 1;
        }
    }
    // 最后一个参数
    if arg_start != command_line.len() {
        args.push(
            core::str::from_utf8(&command_line[arg_start..])
                .unwrap()
                .to_string(),
        );
    }
    args
}

/// 执行运行所有测例的任务
pub fn run_testcases() {
    debug!("run_testcases");
    loop {
        let ans = TESTITER.lock().next().map_or_else(
            || {
                // 已经执行完所有测例，输出测试结果并且跳出
                TESTRESULT.lock().show_result();
                None
            },
            |&command_line| {
                // 清空分配器

                let args = get_args(command_line.as_bytes());
                let testcase = args[0].clone();
                let main_task = Process::new(args).unwrap();
                let now_process_id = main_task.get_process_id() as isize;

                TESTRESULT.lock().load(&(testcase));
                RUN_QUEUE.lock().add_task(main_task);
                let mut exit_code = 0;
                loop {
                    if wait_pid(now_process_id, &mut exit_code as *mut i32).is_ok() {
                        break Some(exit_code);
                    }
                    yield_now_task();
                }
            },
        );
        TaskId::clear();
        unsafe {
            write_page_table_root(KERNEL_PAGE_TABLE.root_paddr());
            asm::sfence_vma_all();
        };
        EXITED_TASKS.lock().clear();
        if let Some(exit_code) = ans {
            let kernel_process = Arc::clone(PID2PC.lock().get(&KERNEL_PROCESS_ID).unwrap());
            kernel_process
                .inner
                .lock()
                .children
                .retain(|x| x.pid == KERNEL_PROCESS_ID);
            // 去除指针引用，此时process_id对应的进程已经被释放
            // 释放所有非内核进程
            finish_one_test(exit_code);
        } else {
            // 已经测试完所有的测例
            break;
        }
        // chdir会改变当前目录，需要重新设置
        api::set_current_dir("/").expect("reset current dir failed");
    }
    panic!("All test finish!");
}

pub fn run_testcase(args: Vec<String>) -> AxResult<()> {
    let main_task = Process::new(args)?;
    RUN_QUEUE.lock().add_task(main_task);
    Ok(())
}
