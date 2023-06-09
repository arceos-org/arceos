use std::env;

pub fn build_zlib_ng(target: &str, compat: bool) {
    let mut cmake = cmake::Config::new("src/zlib-ng");
    cmake
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("ZLIB_COMPAT", if compat { "ON" } else { "OFF" })
        .define("ZLIB_ENABLE_TESTS", "OFF")
        .define("WITH_GZFILEOP", "ON");
    if target.contains("s390x") {
        // Enable hardware compression on s390x.
        cmake
            .define("WITH_DFLTCC_DEFLATE", "1")
            .define("WITH_DFLTCC_INFLATE", "1")
            .cflag("-DDFLTCC_LEVEL_MASK=0x7e");
    }
    if target == "i686-pc-windows-msvc" {
        cmake.define("CMAKE_GENERATOR_PLATFORM", "Win32");
    }

    let install_dir = cmake.build();

    let includedir = install_dir.join("include");
    let libdir = install_dir.join("lib");
    println!(
        "cargo:rustc-link-search=native={}",
        libdir.to_str().unwrap()
    );
    let mut debug_suffix = "";
    let libname = if target.contains("windows") && target.contains("msvc") {
        if env::var("OPT_LEVEL").unwrap() == "0" {
            debug_suffix = "d";
        }
        "zlibstatic"
    } else {
        "z"
    };
    println!(
        "cargo:rustc-link-lib=static={}{}{}",
        libname,
        if compat { "" } else { "-ng" },
        debug_suffix,
    );
    println!("cargo:root={}", install_dir.to_str().unwrap());
    println!("cargo:include={}", includedir.to_str().unwrap());
    if !compat {
        println!("cargo:rustc-cfg=zng");
    }
}

#[allow(dead_code)]
fn main() {
    let target = env::var("TARGET").unwrap();
    build_zlib_ng(&target, false);
}
