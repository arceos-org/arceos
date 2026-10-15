use std::{env, fs, path::PathBuf};

fn main() {
    let source = PathBuf::from("../../src/malloc.rs");
    println!("cargo:rerun-if-changed={}", source.display());
    let source = fs::read_to_string(source).expect("read upstream malloc.rs");
    assert!(
        source.contains("#[repr(C, align(8))]"),
        "probe assumes baseline 8-byte header"
    );
    // Change symbols and include-incompatible doc comments, preserving logic.
    let source = source
        .replace("//!", "//")
        .replace("fn malloc(", "fn probe_malloc(")
        .replace("fn free(", "fn probe_free(");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("upstream_malloc.rs");
    fs::write(output, source).unwrap();
}
