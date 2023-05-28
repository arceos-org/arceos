use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    //to run user mode test, change the mode to "cc"
    //Command::new("cc")
    Command::new("riscv64-linux-gnu-gcc")
    //Command::new("riscv64-linux-musl-gcc")
        .args(["src/tlsf.c", "-O3", "-c", "-fPIC", "-o"])
        .arg(&format!("{}/tlsf.o", out_dir))
        .status()
        .unwrap();

    //to run user mode test, change the mode to "ar"
    //Command::new("ar")
    Command::new("riscv64-linux-gnu-ar")
    //Command::new("riscv64-linux-musl-ar")
        .args(["crus", "libtlsf.a", "tlsf.o"])
        .current_dir(Path::new(&out_dir))
        .status()
        .unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=tlsf");
}
