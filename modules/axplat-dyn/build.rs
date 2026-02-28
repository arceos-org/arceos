use std::{fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ld = include_str!("link.ld");
    println!("cargo:rustc-link-search={}", out_dir.display());
    let ld_content = ld.replace("{{SMP}}", &format!("{}", 16));
    fs::write(out_dir.join("axplat.x"), ld_content).unwrap();
}
