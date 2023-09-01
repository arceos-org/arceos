extern crate alloc;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use lazy_init::LazyInit;
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
    "./runtest.exe -w entry-static.exe argv",
    "./runtest.exe -w entry-static.exe basename",
    "./runtest.exe -w entry-static.exe clocale_mbfuncs",
    "./runtest.exe -w entry-static.exe clock_gettime",
    "./runtest.exe -w entry-static.exe crypt",
    "./runtest.exe -w entry-static.exe dirname",
    "./runtest.exe -w entry-static.exe env",
    "./runtest.exe -w entry-static.exe fdopen",
    "./runtest.exe -w entry-static.exe fnmatch",
    "./runtest.exe -w entry-static.exe fscanf",
    "./runtest.exe -w entry-static.exe fwscanf",
    "./runtest.exe -w entry-static.exe iconv_open",
    "./runtest.exe -w entry-static.exe inet_pton",
    "./runtest.exe -w entry-static.exe mbc",
    "./runtest.exe -w entry-static.exe memstream",
    "./runtest.exe -w entry-static.exe pthread_cancel_points",
    "./runtest.exe -w entry-static.exe pthread_cancel",
    "./runtest.exe -w entry-static.exe pthread_cond",
    "./runtest.exe -w entry-static.exe pthread_tsd",
    "./runtest.exe -w entry-static.exe qsort",
    "./runtest.exe -w entry-static.exe random",
    "./runtest.exe -w entry-static.exe search_hsearch",
    "./runtest.exe -w entry-static.exe search_insque",
    "./runtest.exe -w entry-static.exe search_lsearch",
    "./runtest.exe -w entry-static.exe search_tsearch",
    "./runtest.exe -w entry-static.exe setjmp",
    "./runtest.exe -w entry-static.exe snprintf",
    "./runtest.exe -w entry-static.exe socket",
    "./runtest.exe -w entry-static.exe sscanf",
    "./runtest.exe -w entry-static.exe sscanf_long",
    "./runtest.exe -w entry-static.exe stat",
    "./runtest.exe -w entry-static.exe strftime",
    "./runtest.exe -w entry-static.exe string",
    "./runtest.exe -w entry-static.exe string_memcpy",
    "./runtest.exe -w entry-static.exe string_memmem",
    "./runtest.exe -w entry-static.exe string_memset",
    "./runtest.exe -w entry-static.exe string_strchr",
    "./runtest.exe -w entry-static.exe string_strcspn",
    "./runtest.exe -w entry-static.exe string_strstr",
    "./runtest.exe -w entry-static.exe strptime",
    "./runtest.exe -w entry-static.exe strtod",
    "./runtest.exe -w entry-static.exe strtod_simple",
    "./runtest.exe -w entry-static.exe strtof",
    "./runtest.exe -w entry-static.exe strtol",
    "./runtest.exe -w entry-static.exe strtold",
    "./runtest.exe -w entry-static.exe swprintf",
    "./runtest.exe -w entry-static.exe tgmath",
    "./runtest.exe -w entry-static.exe time",
    "./runtest.exe -w entry-static.exe tls_align",
    "./runtest.exe -w entry-static.exe udiv",
    "./runtest.exe -w entry-static.exe ungetc",
    "./runtest.exe -w entry-static.exe utime",
    "./runtest.exe -w entry-static.exe wcsstr",
    "./runtest.exe -w entry-static.exe wcstol",
    "./runtest.exe -w entry-static.exe pleval",
    "./runtest.exe -w entry-static.exe daemon_failure",
    "./runtest.exe -w entry-static.exe dn_expand_empty",
    "./runtest.exe -w entry-static.exe dn_expand_ptr_0",
    "./runtest.exe -w entry-static.exe fflush_exit",
    "./runtest.exe -w entry-static.exe fgets_eof",
    "./runtest.exe -w entry-static.exe fgetwc_buffering",
    "./runtest.exe -w entry-static.exe fpclassify_invalid_ld80",
    "./runtest.exe -w entry-static.exe ftello_unflushed_append",
    "./runtest.exe -w entry-static.exe getpwnam_r_crash",
    "./runtest.exe -w entry-static.exe getpwnam_r_errno",
    "./runtest.exe -w entry-static.exe iconv_roundtrips",
    "./runtest.exe -w entry-static.exe inet_ntop_v4mapped",
    "./runtest.exe -w entry-static.exe inet_pton_empty_last_field",
    "./runtest.exe -w entry-static.exe iswspace_null",
    "./runtest.exe -w entry-static.exe lrand48_signextend",
    "./runtest.exe -w entry-static.exe lseek_large",
    "./runtest.exe -w entry-static.exe malloc_0",
    "./runtest.exe -w entry-static.exe mbsrtowcs_overflow",
    "./runtest.exe -w entry-static.exe memmem_oob_read",
    "./runtest.exe -w entry-static.exe memmem_oob",
    "./runtest.exe -w entry-static.exe mkdtemp_failure",
    "./runtest.exe -w entry-static.exe mkstemp_failure",
    "./runtest.exe -w entry-static.exe printf_1e9_oob",
    "./runtest.exe -w entry-static.exe printf_fmt_g_round",
    "./runtest.exe -w entry-static.exe printf_fmt_g_zeros",
    "./runtest.exe -w entry-static.exe printf_fmt_n",
    "./runtest.exe -w entry-static.exe pthread_robust_detach",
    "./runtest.exe -w entry-static.exe pthread_cancel_sem_wait",
    "./runtest.exe -w entry-static.exe pthread_cond_smasher",
    "./runtest.exe -w entry-static.exe pthread_condattr_setclock",
    "./runtest.exe -w entry-static.exe pthread_exit_cancel",
    "./runtest.exe -w entry-static.exe pthread_once_deadlock",
    "./runtest.exe -w entry-static.exe pthread_rwlock_ebusy",
    "./runtest.exe -w entry-static.exe putenv_doublefree",
    "./runtest.exe -w entry-static.exe regex_backref_0",
    "./runtest.exe -w entry-static.exe regex_bracket_icase",
    "./runtest.exe -w entry-static.exe regex_ere_backref",
    "./runtest.exe -w entry-static.exe regex_escaped_high_byte",
    "./runtest.exe -w entry-static.exe regex_negated_range",
    "./runtest.exe -w entry-static.exe regexec_nosub",
    "./runtest.exe -w entry-static.exe rewind_clear_error",
    "./runtest.exe -w entry-static.exe rlimit_open_files",
    "./runtest.exe -w entry-static.exe scanf_bytes_consumed",
    "./runtest.exe -w entry-static.exe scanf_match_literal_eof",
    "./runtest.exe -w entry-static.exe scanf_nullbyte_char",
    "./runtest.exe -w entry-static.exe setvbuf_unget",
    "./runtest.exe -w entry-static.exe sigprocmask_internal",
    "./runtest.exe -w entry-static.exe sscanf_eof",
    "./runtest.exe -w entry-static.exe statvfs",
    "./runtest.exe -w entry-static.exe strverscmp",
    "./runtest.exe -w entry-static.exe syscall_sign_extend",
    "./runtest.exe -w entry-static.exe uselocale_0",
    "./runtest.exe -w entry-static.exe wcsncpy_read_overflow",
    "./runtest.exe -w entry-static.exe wcsstr_false_negative",
];

/// 来自 libc 的动态测例
#[allow(dead_code)]
pub const LIBC_DYNAMIC_TESTCASES: &[&str] = &[
    "./runtest.exe -w entry-dynamic.exe argv.dout",
    "./runtest.exe -w entry-dynamic.exe basename.dout",
    "./runtest.exe -w entry-dynamic.exe clocale_mbfuncs.dout",
    "./runtest.exe -w entry-dynamic.exe clock_gettime.dout",
    "./runtest.exe -w entry-dynamic.exe crypt.dout",
    "./runtest.exe -w entry-dynamic.exe dirname.dout",
    "./runtest.exe -w entry-dynamic.exe dlopen.dout", // 单独存在运行时bug，放在runtest里面就是正常的
    "./runtest.exe -w entry-dynamic.exe dlopen",
    "./runtest.exe -w entry-dynamic.exe env.dout",
    "./runtest.exe -w entry-dynamic.exe fdopen.dout",
    "./runtest.exe -w entry-dynamic.exe fnmatch.dout",
    "./runtest.exe -w entry-dynamic.exe fscanf.dout",
    "./runtest.exe -w entry-dynamic.exe fwscanf.dout",
    "./runtest.exe -w entry-dynamic.exe iconv_open.dout",
    "./runtest.exe -w entry-dynamic.exe inet_pton.dout",
    "./runtest.exe -w entry-dynamic.exe mbc.dout",
    "./runtest.exe -w entry-dynamic.exe memstream.dout",
    "./runtest.exe -w entry-dynamic.exe pthread_cancel_points.dout",
    "./runtest.exe -w entry-dynamic.exe pthread_cancel.dout",
    "./runtest.exe -w entry-dynamic.exe pthread_cond.dout",
    "./runtest.exe -w entry-dynamic.exe pthread_tsd.dout",
    "./runtest.exe -w entry-dynamic.exe qsort.dout",
    "./runtest.exe -w entry-dynamic.exe random.dout",
    "./runtest.exe -w entry-dynamic.exe search_hsearch.dout",
    "./runtest.exe -w entry-dynamic.exe search_insque.dout",
    "./runtest.exe -w entry-dynamic.exe search_lsearch.dout",
    "./runtest.exe -w entry-dynamic.exe search_tsearch.dout",
    "./runtest.exe -w entry-dynamic.exe sem_init.dout",
    "./runtest.exe -w entry-dynamic.exe setjmp.dout",
    "./runtest.exe -w entry-dynamic.exe snprintf.dout",
    "./runtest.exe -w entry-dynamic.exe socket",
    "./runtest.exe -w entry-dynamic.exe sscanf.dout",
    "./runtest.exe -w entry-dynamic.exe sscanf_long.dout",
    "./runtest.exe -w entry-dynamic.exe stat.dout",
    "./runtest.exe -w entry-dynamic.exe strftime.dout",
    "./runtest.exe -w entry-dynamic.exe string.dout",
    "./runtest.exe -w entry-dynamic.exe string_memcpy.dout",
    "./runtest.exe -w entry-dynamic.exe string_memmem.dout",
    "./runtest.exe -w entry-dynamic.exe string_memset.dout",
    "./runtest.exe -w entry-dynamic.exe string_strchr.dout",
    "./runtest.exe -w entry-dynamic.exe string_strcspn.dout",
    "./runtest.exe -w entry-dynamic.exe string_strstr.dout",
    "./runtest.exe -w entry-dynamic.exe strptime.dout",
    "./runtest.exe -w entry-dynamic.exe strtod.dout",
    "./runtest.exe -w entry-dynamic.exe strtod_simple.dout",
    "./runtest.exe -w entry-dynamic.exe strtof.dout",
    "./runtest.exe -w entry-dynamic.exe strtol.dout",
    "./runtest.exe -w entry-dynamic.exe strtold.dout",
    "./runtest.exe -w entry-dynamic.exe swprintf.dout",
    "./runtest.exe -w entry-dynamic.exe tgmath.dout",
    "./runtest.exe -w entry-dynamic.exe time.dout",
    "./runtest.exe -w entry-dynamic.exe tls_init.dout",
    "./runtest.exe -w entry-dynamic.exe tls_local_exec.dout",
    "./runtest.exe -w entry-dynamic.exe udiv.dout",
    "./runtest.exe -w entry-dynamic.exe ungetc.dout",
    "./runtest.exe -w entry-dynamic.exe utime.dout",
    "./runtest.exe -w entry-dynamic.exe wcsstr.dout",
    "./runtest.exe -w entry-dynamic.exe wcstol.dout",
    "./runtest.exe -w entry-dynamic.exe daemon_failure.dout",
    "./runtest.exe -w entry-dynamic.exe dn_expand_empty.dout",
    "./runtest.exe -w entry-dynamic.exe dn_expand_ptr_0.dout",
    "./runtest.exe -w entry-dynamic.exe fflush_exit.dout",
    "./runtest.exe -w entry-dynamic.exe fgets_eof.dout",
    "./runtest.exe -w entry-dynamic.exe fgetwc_buffering.dout",
    "./runtest.exe -w entry-dynamic.exe fpclassify_invalid_ld80.dout",
    "./runtest.exe -w entry-dynamic.exe ftello_unflushed_append.dout",
    "./runtest.exe -w entry-dynamic.exe getpwnam_r_crash.dout",
    "./runtest.exe -w entry-dynamic.exe getpwnam_r_errno.dout",
    "./runtest.exe -w entry-dynamic.exe iconv_roundtrips.dout",
    "./runtest.exe -w entry-dynamic.exe inet_ntop_v4mapped.dout",
    "./runtest.exe -w entry-dynamic.exe inet_pton_empty_last_field.dout",
    "./runtest.exe -w entry-dynamic.exe iswspace_null.dout",
    "./runtest.exe -w entry-dynamic.exe lrand48_signextend.dout",
    "./runtest.exe -w entry-dynamic.exe lseek_large.dout",
    "./runtest.exe -w entry-dynamic.exe malloc_0.dout",
    "./runtest.exe -w entry-dynamic.exe mbsrtowcs_overflow.dout",
    "./runtest.exe -w entry-dynamic.exe memmem_oob_read.dout",
    "./runtest.exe -w entry-dynamic.exe memmem_oob.dout",
    "./runtest.exe -w entry-dynamic.exe mkdtemp_failure.dout",
    "./runtest.exe -w entry-dynamic.exe mkstemp_failure.dout",
    "./runtest.exe -w entry-dynamic.exe printf_1e9_oob.dout",
    "./runtest.exe -w entry-dynamic.exe printf_fmt_g_round.dout",
    "./runtest.exe -w entry-dynamic.exe printf_fmt_g_zeros.dout",
    "./runtest.exe -w entry-dynamic.exe printf_fmt_n.dout",
    "./runtest.exe -w entry-dynamic.exe pthread_robust_detach.dout",
    "./runtest.exe -w entry-dynamic.exe pthread_cond_smasher.dout",
    "./runtest.exe -w entry-dynamic.exe pthread_condattr_setclock.dout",
    "./runtest.exe -w entry-dynamic.exe pthread_exit_cancel.dout",
    "./runtest.exe -w entry-dynamic.exe pthread_once_deadlock.dout",
    "./runtest.exe -w entry-dynamic.exe pthread_rwlock_ebusy.dout",
    "./runtest.exe -w entry-dynamic.exe putenv_doublefree.dout",
    "./runtest.exe -w entry-dynamic.exe regex_backref_0.dout",
    "./runtest.exe -w entry-dynamic.exe regex_bracket_icase.dout",
    "./runtest.exe -w entry-dynamic.exe regex_ere_backref.dout",
    "./runtest.exe -w entry-dynamic.exe regex_escaped_high_byte.dout",
    "./runtest.exe -w entry-dynamic.exe regex_negated_range.dout",
    "./runtest.exe -w entry-dynamic.exe regexec_nosub.dout",
    "./runtest.exe -w entry-dynamic.exe rewind_clear_error.dout",
    "./runtest.exe -w entry-dynamic.exe rlimit_open_files.dout",
    "./runtest.exe -w entry-dynamic.exe scanf_bytes_consumed.dout",
    "./runtest.exe -w entry-dynamic.exe scanf_match_literal_eof.dout",
    "./runtest.exe -w entry-dynamic.exe scanf_nullbyte_char.dout",
    "./runtest.exe -w entry-dynamic.exe setvbuf_unget.dout",
    "./runtest.exe -w entry-dynamic.exe sigprocmask_internal.dout",
    "./runtest.exe -w entry-dynamic.exe sscanf_eof.dout",
    "./runtest.exe -w entry-dynamic.exe statvfs.dout",
    "./runtest.exe -w entry-dynamic.exe strverscmp.dout",
    "./runtest.exe -w entry-dynamic.exe syscall_sign_extend.dout",
    "./runtest.exe -w entry-dynamic.exe tls_get_new_dtv.dout",
    "./runtest.exe -w entry-dynamic.exe uselocale_0.dout",
    "./runtest.exe -w entry-dynamic.exe wcsncpy_read_overflow.dout",
    "./runtest.exe -w entry-dynamic.exe wcsstr_false_negative.dout",
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
pub const SDCARD_TESTCASES: &[&str] = &[
    // "./time-test",
    "./interrupts-test-1",
    "./interrupts-test-2",
    "./copy-file-range-test-1",
    "./copy-file-range-test-2",
    "./copy-file-range-test-3",
    "./copy-file-range-test-4",
    "busybox echo hello",
    "busybox sh ./unixbench_testcode.sh",
    "./copy-file-range-test-1",
    "./copy-file-range-test-2",
    "./copy-file-range-test-3",
    "./copy-file-range-test-4",
    "busybox echo hello",
    "busybox sh ./iperf_testcode.sh",
    "./interrupts-test-1",
    "./interrupts-test-1",
    "busybox echo hello",
    "busybox sh busybox_testcode.sh",
    "./interrupts-test-2",
    "./interrupts-test-2",
    "busybox echo hello",
    "busybox sh ./iozone_testcode.sh",
    "busybox echo latency measurements",
    "lmbench_all lat_syscall -P 1 null",
    "lmbench_all lat_syscall -P 1 read",
    "lmbench_all lat_syscall -P 1 write",
    "busybox mkdir -p /var/tmp",
    "busybox touch /var/tmp/lmbench",
    "lmbench_all lat_syscall -P 1 stat /var/tmp/lmbench",
    "lmbench_all lat_syscall -P 1 fstat /var/tmp/lmbench",
    "lmbench_all lat_syscall -P 1 open /var/tmp/lmbench",
    "lmbench_all lat_select -n 100 -P 1 file",
    "lmbench_all lat_sig -P 1 install",
    "lmbench_all lat_sig -P 1 catch",
    "lmbench_all lat_sig -P 1 prot lat_sig",
    "lmbench_all lat_pipe -P 1",
    "lmbench_all lat_proc -P 1 fork",
    "lmbench_all lat_proc -P 1 exec",
    "busybox cp hello /tmp",
    "lmbench_all lat_proc -P 1 shell",
    "lmbench_all lmdd label=\"File /var/tmp/XXX write bandwidth:\" of=/var/tmp/XXX move=1m fsync=1 print=3",
    "lmbench_all lat_pagefault -P 1 /var/tmp/XXX",
    "lmbench_all lat_mmap -P 1 512k /var/tmp/XXX",
    "busybox echo file system latency",
    "lmbench_all lat_fs /var/tmp",
    "busybox echo Bandwidth measurements",
    "lmbench_all bw_pipe -P 1",
    "lmbench_all bw_file_rd -P 1 512k io_only /var/tmp/XXX",
    "lmbench_all bw_file_rd -P 1 512k open2close /var/tmp/XXX",
    "lmbench_all bw_mmap_rd -P 1 512k mmap_only /var/tmp/XXX",
    "lmbench_all bw_mmap_rd -P 1 512k open2close /var/tmp/XXX",
    "busybox echo context switch overhead",
    "lmbench_all lat_ctx -P 1 -s 32 2 4 8 16 24 32 64 96",
    "busybox sh libctest_testcode.sh",
    "busybox sh lua_testcode.sh",
    "libc-bench",
    "busybox sh ./netperf_testcode.sh",
];

pub const NETPERF_TESTCASES: &[&str] = &[
    "netperf -H 127.0.0.1 -p 12865 -t UDP_STREAM -l 1 -- -s 16k -S 16k -m 1k -M 1k",
    "netperf -H 127.0.0.1 -p 12865 -t TCP_STREAM -l 1 -- -s 16k -S 16k -m 1k -M 1k",
    "netperf -H 127.0.0.1 -p 12865 -t UDP_RR -l 1 -- -s 16k -S 16k -m 1k -M 1k -r 64,64 -R 1",
    "netperf -H 127.0.0.1 -p 12865 -t TCP_RR -l 1 -- -s 16k -S 16k -m 1k -M 1k -r 64,64 -R 1",
    "netperf -H 127.0.0.1 -p 12865 -t TCP_CRR -l 1 -- -s 16k -S 16k -m 1k -M 1k -r 64,64 -R 1",
];

pub const IPERF_TESTCASES: &[&str] = &[
    "iperf3 -c 127.0.0.1 -p 5001 -t 2 -i 0", // basic tcp
    "iperf3 -c 127.0.0.1 -p 5001 -t 2 -i 0 -u -b 100G", // basic udp
    "iperf3 -c 127.0.0.1 -p 5001 -t 2 -i 0 -P 5", // parallel tcp
    "iperf3 -c 127.0.0.1 -p 5001 -t 2 -i 0 -u -P 5 -b 1000G", // parallel udp
    "iperf3 -c 127.0.0.1 -p 5001 -t 2 -i 0 -R", // reverse tcp
    "iperf3 -c 127.0.0.1 -p 5001 -t 2 -i 0 -u -R -b 1000G", // reverse udp
];

#[allow(unused)]
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

pub fn run_testcases(case: &'static str) {
    axlog::ax_println!("run_testcases :{}", case);
    let test_iter: LazyInit<Box<dyn Iterator<Item = &'static &'static str> + Send>> =
        LazyInit::new();

    match case {
        "junior" => {
            test_iter.init_by(Box::new(JUNIOR_TESTCASES.iter()));
            // TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(JUNIOR_TESTCASES.len())));
        }
        "libc-static" => {
            test_iter.init_by(Box::new(LIBC_STATIC_TESTCASES.iter()));
            // TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(LIBC_STATIC_TESTCASES.len())));
        }
        "libc-dynamic" => {
            test_iter.init_by(Box::new(LIBC_DYNAMIC_TESTCASES.iter()));
            // TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(
            // LIBC_DYNAMIC_TESTCASES.len(),
            // )));
        }
        "lua" => {
            test_iter.init_by(Box::new(LUA_TESTCASES.iter()));
            // TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(LUA_TESTCASES.len())));
        }
        "netperf" => test_iter.init_by(Box::new(NETPERF_TESTCASES.iter())),

        "ipref" => test_iter.init_by(Box::new(IPERF_TESTCASES.iter())),

        "sdcard" => {
            test_iter.init_by(Box::new(SDCARD_TESTCASES.iter()));
            // TESTRESULT.init_by(SpinNoIrq::new(TestResult::new(BUSYBOX_TESTCASES.len())));
        }
        _ => {
            panic!("unknown test case: {}", case);
        }
    };
    // loop {
    //     let mut ans = None;
    //     if let Some(command_line) = test_iter.next() {
    //         let args = get_args(command_line.as_bytes());
    //         axlog::ax_println!("run newtestcase: {:?}", args);
    //         let testcase = args.clone();
    //         let real_testcase = if testcase[0] == "./busybox".to_string()
    //             || testcase[0] == "busybox".to_string()
    //             || testcase[0] == "entry-static.exe".to_string()
    //             || testcase[0] == "entry-dynamic.exe".to_string()
    //             || testcase[0] == "lmbench_all".to_string()
    //         {
    //             testcase[1].clone()
    //         } else {
    //             testcase[0].clone()
    //         };
    //         // filter(real_testcase);

    //         let main_task = axprocess::Process::new(args).unwrap();
    //         let now_process_id = main_task.get_process_id() as isize;
    //         TESTRESULT.lock().load(&(testcase));
    //         RUN_QUEUE.lock().add_task(main_task);
    //         let mut exit_code = 0;
    //         ans = loop {
    //             if wait_pid(now_process_id, &mut exit_code as *mut i32).is_ok() {
    //                 break Some(exit_code);
    //             }
    //             // let trap: usize = 0xFFFFFFC0805BFEF8;
    //             // let trap_frame: *const TrapFrame = trap as *const TrapFrame;
    //             // info!("trap_frame: {:?}", unsafe { &*trap_frame });
    //             yield_now_task();
    //             // axhal::arch::enable_irqs();
    //         };
    //     }
    //     TaskId::clear();
    //     unsafe {
    //         write_page_table_root(KERNEL_PAGE_TABLE.root_paddr());
    //         asm::sfence_vma_all();
    //     };
    //     EXITED_TASKS.lock().clear();
    //     if let Some(exit_code) = ans {
    //         let kernel_process = Arc::clone(PID2PC.lock().get(&KERNEL_PROCESS_ID).unwrap());
    //         kernel_process
    //             .inner
    //             .lock()
    //             .children
    //             .retain(|x| x.pid == KERNEL_PROCESS_ID);
    //         // 去除指针引用，此时process_id对应的进程已经被释放
    //         // 释放所有非内核进程
    //         finish_one_test(exit_code);
    //     } else {
    //         // 已经测试完所有的测例
    //         TESTRESULT.lock().show_result();
    //         break;
    //     }
    //     // chdir会改变当前目录，需要重新设置
    //     api::set_current_dir("/").expect("reset current dir failed");
    // }
    axlog::ax_println!("hello world!");
}
