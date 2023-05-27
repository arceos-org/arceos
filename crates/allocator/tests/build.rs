use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // test.c
    Command::new("cc")
        .args(&["tests/test.c", "-O3", "-c", "-fPIC", "-o"])
        .arg(&format!("{}/test.o", out_dir))
        .status()
        .unwrap();

    Command::new("ar")
        .args(&["crus", "libtest.a", "test.o"])
        .current_dir(&Path::new(&out_dir))
        .status()
        .unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=test");

    //mitest/mitest.c
    Command::new("cc")
        .args(&["tests/mitest/mitest.c", "-O3", "-c", "-fPIC", "-o"])
        .arg(&format!("{}/mitest.o", out_dir))
        .status()
        .unwrap();
    Command::new("ar")
        .args(&["crus", "libmitest.a", "mitest.o"])
        .current_dir(&Path::new(&out_dir))
        .status()
        .unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=mitest");

    //malloc_large/malloc_large.c
    Command::new("cc")
        .args(&[
            "tests/malloc_large/malloc_large.c",
            "-O3",
            "-c",
            "-fPIC",
            "-o",
        ])
        .arg(&format!("{}/malloc_large.o", out_dir))
        .status()
        .unwrap();
    Command::new("ar")
        .args(&["crus", "libmalloc_large.a", "malloc_large.o"])
        .current_dir(&Path::new(&out_dir))
        .status()
        .unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=malloc_large");

    //glibc_bench/glibc_bench.c
    Command::new("cc")
        .args(&[
            "tests/glibc_bench/glibc_bench.c",
            "-O3",
            "-c",
            "-fPIC",
            "-o",
        ])
        .arg(&format!("{}/glibc_bench.o", out_dir))
        .status()
        .unwrap();
    Command::new("ar")
        .args(&["crus", "libglibc_bench.a", "glibc_bench.o"])
        .current_dir(&Path::new(&out_dir))
        .status()
        .unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=glibc_bench");
}
