// build.rs

use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // Note that there are a number of downsides to this approach, the comments
    // below detail how to improve the portability of these commands.
    Command::new("riscv64-linux-musl-gcc").args(&["src/c_main.c", "-c", "-fPIC", "-o"])
                       .arg(&format!("{}/c_main.o", out_dir))
                       .status().unwrap();
    Command::new("riscv64-linux-musl-ar").args(&["crus", "libc_main.a", "c_main.o"])
                      .current_dir(&Path::new(&out_dir))
                      .status().unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=c_main");
    println!("cargo:rerun-if-changed=src/c_main.c");
}