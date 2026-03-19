use std::{fs, path::PathBuf};

const LINKER_SCRIPT_NAME: &str = "axplat.x";

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ld = include_str!("link.ld");
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-arg=-T{LINKER_SCRIPT_NAME}");
    let ld_content = ld.replace("{{SMP}}", &format!("{}", 16));
    fs::write(out_dir.join(LINKER_SCRIPT_NAME), ld_content).unwrap();
}
