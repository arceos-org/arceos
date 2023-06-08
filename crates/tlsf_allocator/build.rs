fn main() {
    println!("cargo:rustc-link-lib=static=tlsf");
    println!("cargo:rerun-if-changed=src/tlsf.c");
    println!("cargo:rerun-if-changed=src/tlsf.h");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let clippy_args = std::env::var("CLIPPY_ARGS");

    // Not build with clippy or doc
    if target_os == "none" && clippy_args.is_err() {
        let mut base_config = cc::Build::new();
        base_config.file("src/tlsf.c");

        base_config.warnings(true).compile("libtlsf.a");
    }
}
