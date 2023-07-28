use alloc::sync::Arc;
use axfs::{
    api::{self},
    monolithic_fs::flags::OpenFlags,
};
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
use axprocess::{
    link::FilePath,
    process::{wait_pid, yield_now_task, Process, KERNEL_PROCESS_ID, PID2PC},
};
use riscv::asm;

use crate::fs::{file::new_file, link::create_link};

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
    "fdopen",
    "fnmatch",
    "fscanf",
    "fwscanf",
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
    "stat",
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
    "tls_align",
    "udiv",
    "ungetc",
    "utime",
    "wcsstr",
    "wcstol",
    "pleval",
    "daemon_failure",
    "dn_expand_empty",
    "dn_expand_ptr_0",
    "fflush_exit",
    "fgets_eof",
    "fgetwc_buffering",
    "fpclassify_invalid_ld80",
    "ftello_unflushed_append",
    "getpwnam_r_crash",
    "getpwnam_r_errno",
    "iconv_roundtrips",
    "inet_ntop_v4mapped",
    "inet_pton_empty_last_field",
    "iswspace_null",
    "lrand48_signextend",
    "lseek_large",
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
    "rewind_clear_error",
    "rlimit_open_files",
    "scanf_bytes_consumed",
    "scanf_match_literal_eof",
    "scanf_nullbyte_char",
    "setvbuf_unget",
    "sigprocmask_internal",
    "sscanf_eof",
    "statvfs",
    "strverscmp",
    "syscall_sign_extend",
    "uselocale_0",
    "wcsncpy_read_overflow",
    "wcsstr_false_negative",
    // "./runtest.exe -w entry-static.exe pthread_cancel_points",
    // "./runtest.exe -w entry-static.exe pthread_cancel",
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
    // // "dlopen.dout", // 单独存在运行时bug，放在runtest里面就是正常的
    // "./runtest.exe -w entry-dynamic.exe dlopen",
    // "env.dout",
    // "fdopen.dout", // 62
    // "fnmatch.dout",
    // "fscanf.dout",  //62
    // "fwscanf.dout", //29
    // "iconv_open.dout",
    // "inet_pton.dout",
    // "mbc.dout",
    // "memstream.dout",
    // "pthread_cancel_points.dout", // 226
    // "pthread_cancel.dout",        // 226
    // "pthread_cond.dout",          //226
    // "pthread_tsd.dout",           //226
    // "qsort.dout",
    // "random.dout",
    // "search_hsearch.dout",
    // "search_insque.dout",
    // "search_lsearch.dout",
    // "search_tsearch.dout",
    // "sem_init.dout", //226
    // "setjmp.dout",
    // "snprintf.dout",
    "socket", //198
             // "sscanf.dout",
             // "sscanf_long.dout",
             // "stat.dout", //79
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
             // "tls_init.dout",       //226
             // "tls_local_exec.dout", //226
             // "udiv.dout",
             // "ungetc.dout", // 29
             // "utime.dout",  //88
             // "wcsstr.dout",
             // "wcstol.dout",
             // "daemon_failure.dout",
             // "dn_expand_empty.dout",
             // "dn_expand_ptr_0.dout",
             // "fflush_exit.dout", //29 + 67
             // "fgets_eof.dout",
             // "fgetwc_buffering.dout",
             // "fpclassify_invalid_ld80.dout",
             // "ftello_unflushed_append.dout", //25
             // "getpwnam_r_crash.dout",
             // "getpwnam_r_errno.dout",
             // "iconv_roundtrips.dout",
             // "inet_ntop_v4mapped.dout",
             // "inet_pton_empty_last_field.dout",
             // "iswspace_null.dout",
             // "lrand48_signextend.dout",
             // "lseek_large.dout", //29
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
             // "pthread_robust_detach.dout", //226
             // "pthread_cond_smasher.dout",  //226
             // "pthread_condattr_setclock.dout",
             // "pthread_exit_cancel.dout",   //226
             // "pthread_once_deadlock.dout", //226
             // "pthread_rwlock_ebusy.dout",  //226
             // "putenv_doublefree.dout",
             // "regex_backref_0.dout",
             // "regex_bracket_icase.dout",
             // "regex_ere_backref.dout",
             // "regex_escaped_high_byte.dout",
             // "regex_negated_range.dout",
             // "regexec_nosub.dout",
             // "rewind_clear_error.dout", //62
             // "rlimit_open_files.dout",
             // "scanf_bytes_consumed.dout",
             // "scanf_match_literal_eof.dout",
             // "scanf_nullbyte_char.dout",
             // "setvbuf_unget.dout", //62
             // "sigprocmask_internal.dout",
             // "sscanf_eof.dout",
             // "statvfs.dout", //43
             // "strverscmp.dout",
             // "syscall_sign_extend.dout",
             // "tls_get_new_dtv.dout",
             // "uselocale_0.dout",
             // "wcsncpy_read_overflow.dout",
             // "wcsstr_false_negative.dout",
];

#[allow(dead_code)]
pub const LUA_TESTCASES: &[&str] = &[
    // "lua", // 需标准输入，不好进行自动测试
    "lua date.lua",
    "lua file_io.lua",
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
    // "busybox echo iozone automatic measurements",
    // "busybox sh cyclictest_testcode.sh",
    // "busybox echo \"run iozone_testcode.sh\"",
    // "busybox sh ./iozone_testcode.sh",
    // "busybox echo iozone throughput write/read measurements",
    // "iozone -t 4 -i 0 -i 1 -r 1k -s 1m",
    // "busybox echo iozone throughput random-read measurements",
    // "iozone -t 4 -i 0 -i 2 -r 1k -s 1m",
    // "busybox sh ./test_all.sh",
    // "busybox echo \"run libctest_testcode.sh\"",
    "busybox sh unixbench_testcode.sh",
    // "./looper 20 ./multi.sh 16",
    // "./fstime -w -t 20 -b 1024 -m 2000",
    // "./fstime -w -t 20 -b 4096 -m 8000",
    // "./fstime -w -t 20 -b 1024 -m 2000",
    // "./arithoh 10",
    // "./looper 20 ./multi.sh 1",
    // "./looper 20 ./multi.sh 8",
    // "./syscall 10",
    // "./dhry2reg 10",
    // "./looper 20 ./multi.sh 1",
    // "./looper 20 ./multi.sh 8",
    // "./fstime -w -t 20 -b 256 -m 500",
    // "./runtest.exe -w entry-dynamic.exe fscanf",
    // "./libctest_testcode.sh",
    // "busybox echo \"run lua_testcode.sh\"",
    // "./lua_testcode.sh",
    // "lua strings.lua",
    // "busybox echo \"run busybox_testcode.sh\"",
    // "./busybox_testcode.sh",
    // "busybox du",
    // "busybox echo \"#### independent command test\"",
    // "busybox ash -c exit",
    // "busybox sh -c exit",
    // "busybox basename /aaa/bbb",
    // "busybox cal",
    // "busybox clear",
    // "busybox date",
    // "busybox df",
    // "busybox dirname /aaa/bbb",
    // "busybox dmesg",
    // "busybox du",
    // "busybox expr 1 + 1", // 需要29
    // "busybox false",
    // "busybox true",
    // "busybox which ls",
    // "busybox uname",    // 需要29
    // "busybox uptime",   // 需要179
    // "busybox printf \"abc\n\"",
    // "busybox ps",      // 需要179
    // "busybox pwd",     // 需要29
    // "busybox free",    // 需要29
    // "busybox hwclock", // 需要29
    // "busybox kill 10",
    // "busybox ls", // 29
    // "busybox sleep 1",
    // "busybox echo \"#### file opration test\"",
    // "busybox touch test.txt",
    // "busybox echo \"hello world\" > test.txt",
    // "busybox cat test.txt",
    //   "busybox cut -c 3 test.txt",
    //   "busybox od test.txt",
    //   "busybox head test.txt",
    //   "busybox tail test.txt",
    //   "busybox hexdump -C test.txt",
    //   "busybox md5sum test.txt",
    //   "busybox echo \"ccccccc\" >> test.txt",
    //   "busybox echo \"bbbbbbb\" >> test.txt",
    //   "busybox echo \"aaaaaaa\" >> test.txt",
    //   "busybox echo \"2222222\" >> test.txt",
    //   "busybox echo \"1111111\" >> test.txt",
    //   "busybox echo \"bbbbbbb\" >> test.txt",
    //   "busybox sort test.txt | ./busybox uniq",
    // "busybox stat test.txt",
    // "busybox strings test.txt",
    // "busybox wc test.txt",
    // "busybox [ -f test.txt ]",
    // "busybox more test.txt",
    // "busybox rm test.txt",
    // "busybox mkdir test_dir",
    // "busybox mv test_dir test",                   // 需要79
    // "busybox rmdir test",                         // 依赖上一条
    // "busybox grep hello busybox_cmd.txt",         //需要29
    // "busybox cp busybox_cmd.txt busybox_cmd.bak", // 依赖前文
    // "busybox rm busybox_cmd.bak",
    // "busybox find -name \"busybox_cmd.txt\"",
    // "busybox sh busybox echo \"hello\"",

    // "echo latency measurements",
    // "lmbench_all lat_syscall -P 1 null",
    // "busybox sh libctest_testcode.sh",
    // "busybox sh lua_testcode.sh",
    // "busybox sh busybox_testcode.sh",
    // "busybox sh lmbench_testcode.sh",
    // "busybox mkdir -p /var/tmp",
    // "busybox echo latency measurements",
    // "lmbench_all lat_syscall -P 1 null",
    // "lmbench_all lat_syscall -P 1 read",
    // "lmbench_all lat_syscall -P 1 write",
    // "busybox mkdir -p /var/tmp",
    // "busybox touch /var/tmp/lmbench",
    // "lmbench_all lat_syscall -P 1 stat /var/tmp/lmbench",
    // "lmbench_all lat_syscall -P 1 fstat /var/tmp/lmbench",
    // "lmbench_all lat_syscall -P 1 open /var/tmp/lmbench",
    // "lmbench_all lat_select -n 100 -P 1 file",
    // "lmbench_all lat_sig -P 1 install",
    // "lmbench_all lat_sig -P 1 catch",
    // "lmbench_all lat_sig -P 1 prot lat_sig",
    // "lmbench_all lat_pipe -P 1",
    // "lmbench_all lat_proc -P 1 fork",
    // "lmbench_all lat_proc -P 1 exec",
    // "busybox cp hello /tmp",
    // "lmbench_all lat_proc -P 1 shell",
    // "lmbench_all lmdd label=\"File /var/tmp/XXX write bandwidth:\" of=/var/tmp/XXX move=1m fsync=1 print=3",
    // "lmbench_all lat_pagefault -P 1 /var/tmp/XXX",
    // "lmbench_all lat_mmap -P 1 512k /var/tmp/XXX",
    // "busybox echo file system latency",
    // "lmbench_all lat_fs /var/tmp",
    // "busybox echo Bandwidth measurements",
    // "lmbench_all bw_pipe -P 1",
    // "lmbench_all bw_file_rd -P 1 512k io_only /var/tmp/XXX",
    // "lmbench_all bw_file_rd -P 1 512k open2close /var/tmp/XXX",
    // "lmbench_all bw_mmap_rd -P 1 512k mmap_only /var/tmp/XXX",
    // "lmbench_all bw_mmap_rd -P 1 512k open2close /var/tmp/XXX",
    // "busybox echo context switch overhead",
    // "lmbench_all lat_ctx -P 1 -s 32 2 4 8 16 24 32 64 96",
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
pub fn fs_init(case: &'static str) {
    // 需要对libc-dynamic进行特殊处理，因为它需要先加载libc.so
    // 建立一个硬链接

    let libc_so = &"ld-musl-riscv64-sf.so.1";
    let libc_so2 = &"ld-musl-riscv64.so.1"; // 另一种名字的 libc.so，非 libc-test 测例库用

    create_link(
        &(FilePath::new(("/lib/".to_string() + libc_so).as_str()).unwrap()),
        &(FilePath::new("libc.so").unwrap()),
    );
    create_link(
        &(FilePath::new(("/lib/".to_string() + libc_so2).as_str()).unwrap()),
        &(FilePath::new("libc.so").unwrap()),
    );

    let tls_so = &"tls_get_new-dtv_dso.so";
    create_link(
        &(FilePath::new(("/lib/".to_string() + tls_so).as_str()).unwrap()),
        &(FilePath::new("tls_get_new-dtv_dso.so").unwrap()),
    );

    if case == "busybox" {
        create_link(
            &(FilePath::new("./sbin/busybox").unwrap()),
            &(FilePath::new("busybox").unwrap()),
        );
        assert!(create_link(
            &(FilePath::new("./sbin/ls").unwrap()),
            &(FilePath::new("busybox").unwrap()),
        ));
        create_link(
            &(FilePath::new("./ls").unwrap()),
            &(FilePath::new("./bin/busybox").unwrap()),
        );
        create_link(
            &(FilePath::new(".sh").unwrap()),
            &(FilePath::new("./bin/busybox").unwrap()),
        );
        create_link(
            &(FilePath::new("./bin/lmbench_all").unwrap()),
            &(FilePath::new("./lmbench_all").unwrap()),
        );
        create_link(
            &(FilePath::new("./bin/iozone").unwrap()),
            &(FilePath::new("./iozone").unwrap()),
        );
        let _ = new_file("/lat_sig", &(OpenFlags::CREATE | OpenFlags::RDWR));
        // let path = "/lat_sig\0";
        // assert!(syscall_openat(0, path.as_ptr(), 0, 0) > 0);
    }
}

/// 执行运行所有测例的任务
pub fn run_testcases(case: &'static str) {
    debug!("run_testcases :{}", case);
    fs_init(case);
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
