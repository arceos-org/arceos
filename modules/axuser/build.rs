fn main() {
    // in order to force recompile
    println!("cargo:rerun-if-changed=user.elf");
}
