use std::path::PathBuf;

fn main() {
    let mut root_dir = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    root_dir.extend(["..", ".."]);
    let config_path = root_dir.join(".axconfig.toml");
    println!("cargo:rerun-if-changed={}", config_path.display());
}
