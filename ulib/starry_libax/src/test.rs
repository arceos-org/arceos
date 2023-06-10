//! 测试用例

use alloc::sync::Arc;
use axerrno::AxResult;
use axfs::api;
use axhal::arch::write_page_table_root;
use axlog::{debug, info};
use axruntime::KERNEL_PAGE_TABLE;
use axtask::{
    monolithic_task::{EXITED_TASKS, RUN_QUEUE},
    TaskId,
};
use lazy_init::LazyInit;
use spinlock::SpinNoIrq;

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use axprocess::process::{wait_pid, yield_now_task, Process, KERNEL_PROCESS_ID, PID2PC};
use riscv::asm;

use crate::fs::{link::create_link, FilePath};

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
    // "daemon_failure",
    "dirname",
    "dn_expand_empty",
    "dn_expand_ptr_0",
    "env",
    "fdopen",
    "fflush_exit",
    "fgets_eof",
    // "fgetwc_buffering",
    "fnmatch",
    "fpclassify_invalid_ld80",
    // "fscanf",
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

/// 来自 libc 的动态测例
#[allow(dead_code)]
pub const LIBC_DYNAMIC_TESTCASES: &[&str] = &[
    "getpwnam_r_crash.dout",
    "fflush_exit.dout",
    "tls_local_exec.dout",
    "inet_ntop_v4mapped.dout",
    "mkstemp_failure.dout",
    "utime.dout",
    "setjmp.dout",
    "string_memset.dout",
    "time.dout",
    "pthread_cond_smasher.dout",
    "fgetwc_buffering.dout",
    "pthread_rwlock_ebusy.dout",
    "sscanf_long.dout",
    "strptime.dout",
    "dn_expand_empty.dout",
    "wcsstr.dout",
    "search_tsearch.dout",
    "memmem_oob_read.dout",
    "mbc.dout",
    "basename.dout",
    "lrand48_signextend.dout",
    "regex_negated_range.dout",
    "sigprocmask_internal.dout",
    "string.dout",
    // "pthread_cancel.dout",
    // "crypt.dout",
    // "search_hsearch.dout",
    // "clocale_mbfuncs.dout",
    // "regex_bracket_icase.dout",
    // "snprintf.dout",
    // "strverscmp.dout",
    // "sem_init.dout",
    // "random.dout",
    // "strtold.dout",
    // "iswspace_null.dout",
    // "regex_ere_backref.dout",
    // "tls_get_new_dtv.dout",
    // "ftello_unflushed_append.dout",
    // "pthread_tsd.dout",
    // "pthread_exit_cancel.dout",
    // "string_strchr.dout",
    // "printf_fmt_g_zeros.dout",
    // "daemon_failure.dout",
    // "mbsrtowcs_overflow.dout",
    // "strtod_simple.dout",
    // "inet_pton_empty_last_field.dout",
    // "strtol.dout",
    // "fscanf.dout",
    // "tgmath.dout",
    // "ungetc.dout",
    // "dn_expand_ptr_0.dout",
    // "socket.dout",
    // "wcsncpy_read_overflow.dout",
    // "getpwnam_r_errno.dout",
    // "argv.dout",
    // "fpclassify_invalid_ld80.dout",
    // "string_memcpy.dout",
    // "setvbuf_unget.dout",
    // "putenv_doublefree.dout",
    // "pthread_cancel_points.dout",
    // "search_insque.dout",
    // "scanf_bytes_consumed.dout",
    // "dirname.dout",
    // "string_strcspn.dout",
    // "clock_gettime.dout",
    // "wcstol.dout",
    // "fdopen.dout",
    // "scanf_match_literal_eof.dout",
    // "sscanf_eof.dout",
    // "pthread_once_deadlock.dout",
    // "fwscanf.dout",
    // "env.dout",
    // "mkdtemp_failure.dout",
    // "fnmatch.dout",
    // "strftime.dout",
    // "wcsstr_false_negative.dout",
    // "syscall_sign_extend.dout",
    // "swprintf.dout",
    // "tls_init.dout",
    // "regexec_nosub.dout",
    // "string_strstr.dout",
    // "scanf_nullbyte_char.dout",
    // "regex_escaped_high_byte.dout",
    // "printf_fmt_g_round.dout",
    // "pthread_cond.dout",
    // "stat.dout",
    // "sscanf.dout",
    // "dlopen.dout",
    // "printf_fmt_n.dout",
    // "uselocale_0.dout",
    // "regex_backref_0.dout",
    // "qsort.dout",
    // "pthread_condattr_setclock.dout",
    // "inet_pton.dout",
    // "search_lsearch.dout",
    // "strtod.dout",
    // "memmem_oob.dout",
    // "string_memmem.dout",
    // "fgets_eof.dout",
    // "rlimit_open_files.dout",
    // "strtof.dout",
    // "memstream.dout",
    // "udiv.dout",
    // "malloc_0.dout",
    // "printf_1e9_oob.dout",
    // "pthread_robust_detach.dout",
    // "rewind_clear_error.dout",
    // "iconv_roundtrips.dout",
    // "lseek_large.dout",
    // "statvfs.dout",
    // "iconv_open.dout",
];

/// 运行测试时的状态机，记录测试结果与内容
struct TestResult {
    /// 测试用例总数
    sum: usize,
    /// 通过的测试用例数
    accepted: usize,
    /// 当前正在测试的测试用例
    now_testcase: Option<String>,
    /// 同时记录名称与进程号
    failed_testcases: Vec<String>,
}

impl TestResult {
    /// 新建一个测试结果
    pub fn new(case_num: usize) -> Self {
        Self {
            sum: case_num,
            accepted: 0,
            now_testcase: None,
            failed_testcases: Vec::new(),
        }
    }
    /// 加载一个测试用例
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

/// 测试用例迭代器
static TESTITER: LazyInit<SpinNoIrq<Box<dyn Iterator<Item = &'static &'static str> + Send>>> =
    LazyInit::new();

/// 测试结果
static TESTRESULT: LazyInit<SpinNoIrq<TestResult>> = LazyInit::new();

/// 某一个测试用例完成之后调用，记录测试结果
pub fn finish_one_test(exit_code: i32) {
    TESTRESULT.lock().finish_one_test(exit_code);
}

/// 展示测试结果
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
pub fn run_testcases(case: &'static str) {
    debug!("run_testcases :{}", case);
    match case {
        "junior" => {
            TESTITER.init_by(SpinNoIrq::new(Box::new(JUNIOR_TESTCASES.iter())));
            TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(JUNIOR_TESTCASES.len())));
        }
        "libc-static" => {
            TESTITER.init_by(SpinNoIrq::new(Box::new(LIBC_STATIC_TESTCASES.iter())));
            TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(LIBC_STATIC_TESTCASES.len())));
        }
        "libc-dyamic" => {
            TESTITER.init_by(SpinNoIrq::new(Box::new(LIBC_DYNAMIC_TESTCASES.iter())));
            TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(
                LIBC_DYNAMIC_TESTCASES.len(),
            )));
            // 需要对libc-dynamic进行特殊处理，因为它需要先加载libc.so
            // 建立一个硬链接
            let libc_so = &"ld-musl-riscv64-sf.so.1";
            let libc_so2 = &"ld-musl-riscv64.so.1"; // 另一种名字的 libc.so，非 libc-test 测例库用

            create_link(
                &FilePath::new(("/lib/".to_string() + libc_so).as_str()),
                &FilePath::new("libc.so"),
            );

            create_link(
                &FilePath::new(("/lib/".to_string() + libc_so2).as_str()),
                &FilePath::new("libc.so"),
            );
            info!("get link!");
        }
        _ => {
            panic!("unknown test case: {}", case);
        }
    };
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

/// 执行单个测例
pub fn run_testcase(args: Vec<String>) -> AxResult<()> {
    let main_task = Process::new(args)?;
    RUN_QUEUE.lock().add_task(main_task);
    Ok(())
}
