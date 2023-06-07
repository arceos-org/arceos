#![no_std]
use libax::{
    process::{exec, fork},
    task::exit,
};

pub fn fake_exec(f: fn()) {
    match fork() {
        pid if pid > 0 => {}
        0 => {
            f();
            exit(0);
        }
        _ => {
            panic!("Error fork()");
        }
    }
}

pub fn real_exec(cmd: &str) {
    match fork() {
        pid if pid > 0 => {}
        0 => exit(exec(cmd) as usize),
        _ => {
            panic!("Error fork()");
        }
    }
}
