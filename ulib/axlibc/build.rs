fn main() {
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
        if let Some(llvm_target) = target.strip_suffix("-softfloat") {
            // remove "-softfloat" suffix for some targets
            builder = builder.clang_arg(format!("--target={llvm_target}"));
        }
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
