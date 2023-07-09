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
    "dirname",
    "env",
    // "fdopen", // 需要62
    "fnmatch",
    // "fscanf", // 需要62
    // "fwscanf", // 需要29
    "iconv_open",
    "inet_pton",
    "mbc",
    "memstream",
    // "pthread_cancel_points", // 需要226
    // "pthread_cancel",        // 需要226
    // "pthread_cond",          // 需要226
    // "pthread_tsd",           // 需要226
    "qsort",
    "random",
    "search_hsearch",
    "search_insque",
    "search_lsearch",
    "search_tsearch",
    "setjmp",
    "snprintf",
    // "socket", // 需要198
    "sscanf",
    "sscanf_long",
    // "stat",        // 需79
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
    "swprintf",
    "tgmath",
    "time",
    // "tls_align", // app不存在
    "udiv",
    // "ungetc", // 需要29
    // "utime",  // 需要88
    "wcsstr",
    "wcstol",
    "pleval",
    "daemon_failure",
    "dn_expand_empty",
    "dn_expand_ptr_0",
    // "fflush_exit", // 需要29
    "fgets_eof",
    "fgetwc_buffering",
    "fpclassify_invalid_ld80",
    // "ftello_unflushed_append", // 需要25
    "getpwnam_r_crash",
    "getpwnam_r_errno",
    "iconv_roundtrips",
    "inet_ntop_v4mapped",
    "inet_pton_empty_last_field",
    "iswspace_null",
    "lrand48_signextend",
    // "lseek_large", // 需要29
    "malloc_0",
    "mbsrtowcs_overflow",
    "memmem_oob_read",
    "memmem_oob",
    "mkdtemp_failure",
    "mkstemp_failure",
    "printf_1e9_oob",
    "printf_fmt_g_round",
    "printf_fmt_g_zeros",
    "printf_fmt_n",
    // "pthread_robust_detach", // 需226
    // "pthread_cancel_sem_wait",   // 需要226
    // "pthread_cond_smasher",      // 需要226
    "pthread_condattr_setclock",
    // "pthread_exit_cancel",       // 需要226
    // "pthread_once_deadlock",     // 需要226
    // "pthread_rwlock_ebusy",      // 需要226
    "putenv_doublefree",
    "regex_backref_0",
    "regex_bracket_icase",
    "regex_ere_backref",
    "regex_escaped_high_byte",
    "regex_negated_range",
    "regexec_nosub",
    // "rewind_clear_error", // 需要62
    "rlimit_open_files",
    "scanf_bytes_consumed",
    "scanf_match_literal_eof",
    "scanf_nullbyte_char",
    // "setvbuf_unget", // 需要62
    "sigprocmask_internal",
    "sscanf_eof",
    // "statvfs", // 需要43
    "strverscmp",
    "syscall_sign_extend",
    "uselocale_0",
    "wcsncpy_read_overflow",
    "wcsstr_false_negative",
];

/// 来自 libc 的动态测例
#[allow(dead_code)]
pub const LIBC_DYNAMIC_TESTCASES: &[&str] = &[
    // "argv.dout",
    // "basename.dout",
    // "clocale_mbfuncs.dout",
    // "clock_gettime.dout",
    // "crypt.dout",
    // "dirname.dout",
    // "dlopen.dout", // 存在运行时bug
    // "env.dout",
    "fdopen.dout", // 62
    // "fnmatch.dout",
    "fscanf.dout",  //62
    "fwscanf.dout", //29
    // "iconv_open.dout",
    // "inet_pton.dout",
    // "mbc.dout",
    // "memstream.dout",
    "pthread_cancel_points.dout", // 226
    "pthread_cancel.dout",        // 226
    "pthread_cond.dout",          //226
    "pthread_tsd.dout",           //226
    // "qsort.dout",
    // "random.dout",
    // "search_hsearch.dout",
    // "search_insque.dout",
    // "search_lsearch.dout",
    // "search_tsearch.dout",
    "sem_init.dout", //226
    // "setjmp.dout",
    // "snprintf.dout",
    "socket.dout", //198
    // "sscanf.dout",
    // "sscanf_long.dout",
    "stat.dout", //79
    // "strftime.dout",
    // "string.dout",
    // "string_memcpy.dout",
    // "string_memmem.dout",
    // "string_memset.dout",
    // "string_strchr.dout",
    // "string_strcspn.dout",
    // "string_strstr.dout",
    // "strptime.dout",
    // "strtod.dout",
    // "strtod_simple.dout",
    // "strtof.dout",
    // "strtol.dout",
    // "strtold.dout",
    // "swprintf.dout",
    // "tgmath.dout",
    // "time.dout",
    "tls_init.dout",       //226
    "tls_local_exec.dout", //226
    // "udiv.dout",
    // "ungetc.dout",
    "utime.dout", //88
    // "wcsstr.dout",
    // "wcstol.dout",
    // "daemon_failure.dout",
    // "dn_expand_empty.dout",
    // "dn_expand_ptr_0.dout",
    "fflush_exit.dout", //29 + 67
    // "fgets_eof.dout",
    // "fgetwc_buffering.dout",
    // "fpclassify_invalid_ld80.dout",
    "ftello_unflushed_append.dout", //25
    // "getpwnam_r_crash.dout",
    // "getpwnam_r_errno.dout",
    // "iconv_roundtrips.dout",
    // "inet_ntop_v4mapped.dout",
    // "inet_pton_empty_last_field.dout",
    // "iswspace_null.dout",
    // "lrand48_signextend.dout",
    "lseek_large.dout", //29
    // "malloc_0.dout",
    // "mbsrtowcs_overflow.dout",
    // "memmem_oob_read.dout",
    // "memmem_oob.dout",
    // "mkdtemp_failure.dout",
    // "mkstemp_failure.dout",
    // "printf_1e9_oob.dout",
    // "printf_fmt_g_round.dout",
    // "printf_fmt_g_zeros.dout",
    // "printf_fmt_n.dout",
    "pthread_robust_detach.dout", //226
    "pthread_cond_smasher.dout",  //226
    // "pthread_condattr_setclock.dout",
    "pthread_exit_cancel.dout",   //226
    "pthread_once_deadlock.dout", //226
    "pthread_rwlock_ebusy.dout",  //226
    // "putenv_doublefree.dout",
    // "regex_backref_0.dout",
    // "regex_bracket_icase.dout",
    // "regex_ere_backref.dout",
    // "regex_escaped_high_byte.dout",
    // "regex_negated_range.dout",
    // "regexec_nosub.dout",
    "rewind_clear_error.dout", //62
    // "rlimit_open_files.dout",
    // "scanf_bytes_consumed.dout",
    // "scanf_match_literal_eof.dout",
    // "scanf_nullbyte_char.dout",
    "setvbuf_unget.dout", //62
    // "sigprocmask_internal.dout",
    // "sscanf_eof.dout",
    "statvfs.dout", //43
    // "strverscmp.dout",
    // "syscall_sign_extend.dout",
    "tls_get_new_dtv.dout", //226
                            // "uselocale_0.dout",
                            // "wcsncpy_read_overflow.dout",
                            // "wcsstr_false_negative.dout",
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

static TESTRESULT: LazyInit<SpinNoIrq<TestResult>> = LazyInit::new();

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
pub fn run_testcases(case: &'static str) {
    let mut test_iter: LazyInit<Box<dyn Iterator<Item = &'static &'static str> + Send>> =
        LazyInit::new();
    debug!("run_testcases :{}", case);
    match case {
        "junior" => {
            test_iter.init_by(Box::new(JUNIOR_TESTCASES.iter()));
            TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(JUNIOR_TESTCASES.len())));
        }
        "libc-static" => {
            test_iter.init_by(Box::new(LIBC_STATIC_TESTCASES.iter()));
            TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(LIBC_STATIC_TESTCASES.len())));
        }
        "libc-dyamic" => {
            test_iter.init_by(Box::new(LIBC_DYNAMIC_TESTCASES.iter()));
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
        let ans = test_iter.next().map_or_else(
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
