fn main() {
    let mut base_config = cc::Build::new();
    base_config.file("src/tlsf.c");

    base_config.warnings(true).compile("libtlsf.a");

    println!("cargo:rustc-link-lib=static=tlsf");
    println!("cargo:rerun-if-changed=src/tlsf.c");
    println!("cargo:rerun-if-changed=src/tlsf.h");
}
