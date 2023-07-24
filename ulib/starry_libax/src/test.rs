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
    //"fdopen", // 需要62
    "fnmatch",
    //"fscanf",  // 需要62
    //"fwscanf", // 需要29
    "iconv_open",
    "inet_pton",
    "mbc",
    "memstream",
    "pthread_cancel_points",
    "pthread_cancel",
    "pthread_cond",
    "pthread_tsd",
    "qsort",
    "random",
    "search_hsearch",
    "search_insque",
    "search_lsearch",
    "search_tsearch",
    "setjmp",
    "snprintf",
    "socket",
    "sscanf",
    "sscanf_long",
    //"stat", // 需79
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
    //"tls_align", // app不存在
    "udiv",
    //"ungetc", // 需要29
    //"utime",  // 需要88
    "wcsstr",
    "wcstol",
    "pleval",
    "daemon_failure",
    "dn_expand_empty",
    "dn_expand_ptr_0",
    //"fflush_exit", // 需要29
    "fgets_eof",
    "fgetwc_buffering",
    "fpclassify_invalid_ld80",
    //"ftello_unflushed_append", // 需要25
    "getpwnam_r_crash",
    "getpwnam_r_errno",
    "iconv_roundtrips",
    "inet_ntop_v4mapped",
    "inet_pton_empty_last_field",
    "iswspace_null",
    "lrand48_signextend",
    //"lseek_large", // 需要29
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
    "pthread_robust_detach",
    "pthread_cancel_sem_wait",
    "pthread_cond_smasher",
    "pthread_condattr_setclock",
    "pthread_exit_cancel",
    "pthread_once_deadlock",
    "pthread_rwlock_ebusy",
    "putenv_doublefree",
    "regex_backref_0",
    "regex_bracket_icase",
    "regex_ere_backref",
    "regex_escaped_high_byte",
    "regex_negated_range",
    "regexec_nosub",
    //"rewind_clear_error", // 需要62
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
    // "basename",
    // "clocale_mbfuncs",
    // "clock_gettime",
    // "crypt",
    // "dirname",
    // "dlopen.dout", // 存在运行时bug
                   // "env",
                   // "fdopen", // 62
                   // // "fnmatch",
                   // "fscanf",  //62
                   // "fwscanf", //29
                   // // "iconv_open",
                   // // "inet_pton",
                   // // "mbc",
                   // // "memstream",
                   // "pthread_cancel_points", // 226
                   // "pthread_cancel",        // 226
                   // "pthread_cond",          //226
                   // "pthread_tsd",           //226
                   // // "qsort",
                   // // "random",
                   // // "search_hsearch",
                   // // "search_insque",
                   // // "search_lsearch",
                   // // "search_tsearch",
                   // "sem_init", //226
                   // // "setjmp",
                   // // "snprintf",
                   // "socket",
                   // // "sscanf",
                   // // "sscanf_long",
                   // "stat", //79
                   // // "strftime",
                   // // "string",
                   // // "string_memcpy",
                   // // "string_memmem",
                   // // "string_memset",
                   // // "string_strchr",
                   // // "string_strcspn",
                   // // "string_strstr",
                   // // "strptime",
                   // // "strtod",
                   // // "strtod_simple",
                   // // "strtof",
                   // // "strtol",
                   // // "strtold",
                   // // "swprintf",
                   // // "tgmath",
                   // // "time",
                   // "tls_init",       //226
                   // "tls_local_exec", //226
                   // // "udiv",
                   // // "ungetc",
                   // "utime", //88
                   // // "wcsstr",
                   // // "wcstol",
                   // // "daemon_failure",
                   // // "dn_expand_empty",
                   // // "dn_expand_ptr_0",
                   // "fflush_exit", //29 + 67
                   // // "fgets_eof",
                   // // "fgetwc_buffering",
                   // // "fpclassify_invalid_ld80",
                   // "ftello_unflushed_append", //25
                   // // "getpwnam_r_crash",
                   // // "getpwnam_r_errno",
                   // // "iconv_roundtrips",
                   // // "inet_ntop_v4mapped",
                   // // "inet_pton_empty_last_field",
                   // // "iswspace_null",
                   // // "lrand48_signextend",
                   // "lseek_large", //29
                   // // "malloc_0",
                   // // "mbsrtowcs_overflow",
                   // // "memmem_oob_read",
                   // // "memmem_oob",
                   // // "mkdtemp_failure",
                   // // "mkstemp_failure",
                   // // "printf_1e9_oob",
                   // // "printf_fmt_g_round",
                   // // "printf_fmt_g_zeros",
                   // // "printf_fmt_n",
                   // "pthread_robust_detach", //226
                   // "pthread_cond_smasher",  //226
                   // // "pthread_condattr_setclock",
                   // "pthread_exit_cancel",   //226
                   // "pthread_once_deadlock", //226
                   // "pthread_rwlock_ebusy",  //226
                   // // "putenv_doublefree",
                   // // "regex_backref_0",
                   // // "regex_bracket_icase",
                   // // "regex_ere_backref",
                   // // "regex_escaped_high_byte",
                   // // "regex_negated_range",
                   // // "regexec_nosub",
                   // "rewind_clear_error", //62
                   // // "rlimit_open_files",
                   // // "scanf_bytes_consumed",
                   // // "scanf_match_literal_eof",
                   // // "scanf_nullbyte_char",
                   // "setvbuf_unget", //62
                   // // "sigprocmask_internal",
                   // // "sscanf_eof",
                   // "statvfs", //43
                   // // "strverscmp",
                   // // "syscall_sign_extend",
                   // "tls_get_new_dtv", //226
                   // "uselocale_0",
                   // "wcsncpy_read_overflow",
                   // "wcsstr_false_negative",
];

#[allow(dead_code)]
pub const LUA_TESTCASES: &[&str] = &[
    "lua", // 需要29
    "lua date.lua",
    "lua file_io.lua", // 需要29
    "lua max_min.lua",
    "lua random.lua",
    "lua remove.lua",
    "lua round_num.lua",
    "lua sin30.lua",
    "lua strings.lua",
    "lua sort.lua",
];

#[allow(dead_code)]
pub const BUSYBOX_TESTCASES: &[&str] = &[
    //"busybox sh ./busybox_testcode.sh", //最终测例，它包含了下面全部
    "busybox echo \"#### independent command test\"",
    "busybox ash -c exit",
    "busybox sh -c exit",
    "busybox basename /aaa/bbb",
    "busybox cal",
    "busybox clear",
    "busybox date",
    "busybox df",
    "busybox dirname /aaa/bbb",
    "busybox dmesg",
    "busybox du",
    "busybox expr 1 + 1",
    "busybox false",
    "busybox true",
    "busybox which ls",
    "busybox uname",
    "busybox uptime",
    "busybox printf \"abc\n\"",
    "busybox ps",
    "busybox pwd",
    "busybox free",
    "busybox hwclock",
    "busybox kill 10",
    "busybox ls",
    "busybox sleep 1",
    "busybox echo \"#### file opration test\"",
    "busybox touch test.txt",
    "busybox echo \"hello world\" > test.txt",
    "busybox cat test.txt",
    "busybox cut -c 3 test.txt",
    "busybox od test.txt",
    "busybox head test.txt",
    "busybox tail test.txt",
    // "busybox hexdump -C test.txt", // 会要求标准输入，不方便自动测试
    "busybox md5sum test.txt",
    "busybox echo \"ccccccc\" >> test.txt",
    "busybox echo \"bbbbbbb\" >> test.txt",
    "busybox echo \"aaaaaaa\" >> test.txt",
    "busybox echo \"2222222\" >> test.txt",
    "busybox echo \"1111111\" >> test.txt",
    "busybox echo \"bbbbbbb\" >> test.txt",
    "busybox sort test.txt | ./busybox uniq",
    "busybox stat test.txt",
    "busybox strings test.txt",
    "busybox wc test.txt",
    "busybox [ -f test.txt ]",
    "busybox more test.txt",
    "busybox rm test.txt",
    "busybox mkdir test_dir",
    "busybox mv test_dir test",
    "busybox rmdir test",
    "busybox grep hello busybox_cmd.txt",
    "busybox cp busybox_cmd.txt busybox_cmd.bak",
    "busybox rm busybox_cmd.bak",
    "busybox find -name \"busybox_cmd.txt\"",
    "busybox sh lua_testcode.sh",
];

/// 运行测试时的状态机，记录测试结果与内容
struct TestResult {
    sum: usize,
    accepted: usize,
    now_testcase: Option<Vec<String>>,
    // 同时记录名称与进程号
    failed_testcases: Vec<Vec<String>>,
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
    pub fn load(&mut self, testcase: &Vec<String>) {
        info!(
            " --------------- load testcase: {:?} --------------- ",
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
            info!("{:?}", test);
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

/// 在执行系统调用前初始化文件系统
///
/// 包括建立软连接，提前准备好一系列的文件与文件夹
pub fn fs_init(case: &'static str) -> AxResult<()> {
    // create_dir("/dev/")?;
    // create_dir("/lib").unwrap();
    // create_dir("tmp/")?;
    // create_dir("/proc").unwrap();
    // create_dir("./dev/shm").unwrap();
    // create_dir("./dev/./misc")?;

    // create_dir("/sbin/")?;
    // new_file("./dev/misc/rtc", &OpenFlags::CREATE)?;
    // new_file("./lat_sig", &OpenFlags::CREATE)?;
    // new_file("./proc/mounts", &OpenFlags::CREATE)?;
    // new_file("./proc/meminfo", &OpenFlags::CREATE)?;

    if case == "libc-dynamic" {
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

    Ok(())
}

/// 执行运行所有测例的任务
pub fn run_testcases(case: &'static str) {
    debug!("run_testcases :{}", case);
    fs_init(case).unwrap();
    let mut test_iter: LazyInit<Box<dyn Iterator<Item = &'static &'static str> + Send>> =
        LazyInit::new();

    match case {
        "junior" => {
            test_iter.init_by(Box::new(JUNIOR_TESTCASES.iter()));
            TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(JUNIOR_TESTCASES.len())));
        }
        "libc-static" => {
            test_iter.init_by(Box::new(LIBC_STATIC_TESTCASES.iter()));
            TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(LIBC_STATIC_TESTCASES.len())));
        }
        "libc-dynamic" => {
            test_iter.init_by(Box::new(LIBC_DYNAMIC_TESTCASES.iter()));
            TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(
                LIBC_DYNAMIC_TESTCASES.len(),
            )));
        }
        "lua" => {
            test_iter.init_by(Box::new(LUA_TESTCASES.iter()));
            TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(LUA_TESTCASES.len())));
        }
        "busybox" => {
            test_iter.init_by(Box::new(BUSYBOX_TESTCASES.iter()));
            TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(BUSYBOX_TESTCASES.len())));
        }
        _ => {
            panic!("unknown test case: {}", case);
        }
    };
    loop {
        let mut ans = None;
        if let Some(command_line) = test_iter.next() {
            let args = get_args(command_line.as_bytes());
            let testcase = args.clone();
            let main_task = Process::new(args).unwrap();
            let now_process_id = main_task.get_process_id() as isize;
            TESTRESULT.lock().load(&(testcase));
            RUN_QUEUE.lock().add_task(main_task);
            let mut exit_code = 0;
            ans = loop {
                if wait_pid(now_process_id, &mut exit_code as *mut i32).is_ok() {
                    break Some(exit_code);
                }
                // let trap: usize = 0xFFFFFFC0805BFEF8;
                // let trap_frame: *const TrapFrame = trap as *const TrapFrame;
                // info!("trap_frame: {:?}", unsafe { &*trap_frame });
                yield_now_task();
                // axhal::arch::enable_irqs();
            };
        }
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
            TESTRESULT.lock().show_result();
            break;
        }
        // chdir会改变当前目录，需要重新设置
        api::set_current_dir("/").expect("reset current dir failed");
    }
    panic!("All test finish!");
}

// pub fn run_testcase(args: Vec<String>) -> AxResult<()> {
//     let main_task = Process::new(args)?;
//     RUN_QUEUE.lock().add_task(main_task);
//     Ok(())
// }
