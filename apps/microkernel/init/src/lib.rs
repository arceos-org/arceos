#![no_std]
use libax::{process::fork, task::exit};

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
