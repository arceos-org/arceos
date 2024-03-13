//! The userboot of the operating system, which will start the first user process and go into the user mode
#![cfg_attr(not(test), no_std)]
#![no_main]

#[allow(unused)]
use axstarry::{println, recycle_user_process, wait_pid, yield_now_task, Process};

mod batch;
mod fs;

extern crate alloc;
#[allow(unused)]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
/// To get the environment variables of the application
pub fn get_envs() -> Vec<String> {
    let mut envs:Vec<String> = vec![
        "SHLVL=1".into(),
        "PWD=/".into(),
        "GCC_EXEC_PREFIX=/riscv64-linux-musl-native/bin/../lib/gcc/".into(),
        "COLLECT_GCC=./riscv64-linux-musl-native/bin/riscv64-linux-musl-gcc".into(),
        "COLLECT_LTO_WRAPPER=/riscv64-linux-musl-native/bin/../libexec/gcc/riscv64-linux-musl/11.2.1/lto-wrapper".into(),
        "COLLECT_GCC_OPTIONS='-march=rv64gc' '-mabi=lp64d' '-march=rv64imafdc' '-dumpdir' 'a.'".into(),
        "LIBRARY_PATH=/lib/".into(),
        "LD_LIBRARY_PATH=/lib/".into(),
        "LD_DEBUG=files".into(),
    ];
    // read the file "/etc/environment"
    // if exist, then append the content to envs
    // else set the environment variable to default value
    if let Some(environment_vars) = axstarry::read_file("/etc/environment") {
        envs.push(environment_vars);
    } else {
        envs.push("PATH=/usr/sbin:/usr/bin:/sbin:/bin".into());
    }

    envs
}

#[no_mangle]
fn main() {
    fs::fs_init();
    let envs = get_envs();
    #[cfg(feature = "batch")]
    {
        batch::run_batch_testcases(&envs);
        println(format!("System halted with exit code {}", 0).as_str());
    }
    #[cfg(not(feature = "batch"))]
    {
        let init_args = vec!["busybox".to_string(), "sh".to_string()];
        let user_process = Process::init(init_args, &envs).unwrap();
        let now_process_id = user_process.get_process_id() as isize;
        let mut exit_code = 0;
        loop {
            if unsafe { wait_pid(now_process_id, &mut exit_code as *mut i32) }.is_ok() {
                break;
            }

            yield_now_task();
        }

        recycle_user_process();
        println(format!("System halted with exit code {}", exit_code).as_str());
    }
}
