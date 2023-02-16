// build.rs

use std::env;
use std::path::Path;
use std::process::Command;

const C_DIR: &str = "../hello_c";
const C_MAIN_FILE: &str = "src/main.c";
const C_LIB: &str = "user_libc.h";

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let c_main = Path::new(C_DIR).join(C_MAIN_FILE);
    let c_lib = Path::new(C_DIR).join(C_LIB);

    Command::new("riscv64-linux-musl-gcc")
        .args(&[c_main.to_str().unwrap(), "-c", "-fPIC", "-o"])
        .arg(&format!("{}/c_main.o", out_dir))
        .status()
        .unwrap();
    Command::new("riscv64-linux-musl-gcc")
        .args(&[c_lib.to_str().unwrap(), "-c", "-fPIC", "-o"])
        .arg(&format!("{}/c_lib.o", out_dir))
        .status()
        .unwrap();
    Command::new("riscv64-linux-musl-ar")
        .args(&["crus", "libc_lib.a", "c_lib.o", "c_main.o"])
        .current_dir(&Path::new(&out_dir))
        .status()
        .unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=c_lib");
    println!("cargo:rerun-if-changed={}", C_DIR);
}
