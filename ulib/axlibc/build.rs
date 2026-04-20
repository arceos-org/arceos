fn main() {
    fn clang_target_for_bindgen(target: &str) -> String {
        match target {
            // Clang does not accept Rust's `riscv64gc` arch suffix in the target triple.
            "riscv64gc-unknown-none-elf" => "riscv64-unknown-elf".to_string(),
            _ => target.strip_suffix("-softfloat").unwrap_or(target).to_string(),
        }
    }

    fn gen_c_to_rust_bindings(in_file: &str, out_file: &str) {
        println!("cargo:rerun-if-changed={in_file}");

        let target = std::env::var("TARGET").unwrap();
        let allow_types = ["tm", "jmp_buf"];
        let mut builder = bindgen::Builder::default()
            .header(in_file)
            .clang_arg("-I./include")
            .derive_default(true)
            .size_t_is_usize(false)
            .use_core();
        // Always pass an explicit clang target to avoid using host ABI.
        // Some Rust target triples are not accepted by clang and need mapping.
        let clang_target = clang_target_for_bindgen(&target);
        builder = builder.clang_arg(format!("--target={clang_target}"));
        for ty in allow_types {
            builder = builder.allowlist_type(ty);
        }

        builder
            .generate()
            .expect("Unable to generate c->rust bindings")
            .write_to_file(out_file)
            .expect("Couldn't write bindings!");
    }

    gen_c_to_rust_bindings("ctypes.h", "src/libctypes_gen.rs");
}
