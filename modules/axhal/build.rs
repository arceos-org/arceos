use std::io::Result;
use std::path::Path;

const AXTEST_SECTIONS: &str = r#"
    .axtest_desc : ALIGN(8) {
        __axtest_suites_start = .;
        KEEP(*(.axtest_desc))
        __axtest_suites_end = .;
    }

    .axtest_mod_hooks : ALIGN(8) {
        __axtest_mod_hooks_start = .;
        KEEP(*(.axtest_mod_hooks))
        __axtest_mod_hooks_end = .;
    }
"#;

const LLVM_PROFILE_SECTIONS: &str = r#"
    /* LLVM profiling sections */
    __llvm_prf_data : ALIGN(8) {
        PROVIDE(__start___llvm_prf_data = .);
        __llvm_prf_data_start = .;
        KEEP(*(__llvm_prf_data))
        PROVIDE(__stop___llvm_prf_data = .);
        __llvm_prf_data_end = .;
    }

    __llvm_prf_vnds : ALIGN(8) {
        PROVIDE(__start___llvm_prf_vnds = .);
        __llvm_prf_vnds_start = .;
        KEEP(*(__llvm_prf_vnds))
        PROVIDE(__stop___llvm_prf_vnds = .);
        __llvm_prf_vnds_end = .;
    }

    __llvm_prf_vns : ALIGN(8) {
        PROVIDE(__start___llvm_prf_vns = .);
        KEEP(*(__llvm_prf_vns))
        PROVIDE(__stop___llvm_prf_vns = .);
    }

    __llvm_prf_vtab : ALIGN(8) {
        PROVIDE(__start___llvm_prf_vtab = .);
        KEEP(*(__llvm_prf_vtab))
        PROVIDE(__stop___llvm_prf_vtab = .);
    }

    __llvm_prf_names : ALIGN(8) {
        PROVIDE(__start___llvm_prf_names = .);
        __llvm_prf_names_start = .;
        KEEP(*(__llvm_prf_names))
        PROVIDE(__stop___llvm_prf_names = .);
        __llvm_prf_names_end = .;
    }

    __llvm_prf_bits : ALIGN(8) {
        PROVIDE(__start___llvm_prf_bits = .);
        KEEP(*(__llvm_prf_bits))
        PROVIDE(__stop___llvm_prf_bits = .);
    }

    __llvm_prf_cnts : ALIGN(8) {
        PROVIDE(__start___llvm_prf_cnts = .);
        __llvm_prf_cnts_start = .;
        KEEP(*(__llvm_prf_cnts))
        PROVIDE(__stop___llvm_prf_cnts = .);
        __llvm_prf_cnts_end = .;
    }

    __llvm_orderfile : ALIGN(8) {
        PROVIDE(__start___llvm_orderfile = .);
        KEEP(*(__llvm_orderfile))
        PROVIDE(__stop___llvm_orderfile = .);
    }
"#;

fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.as_str(), "1" | "y" | "Y" | "yes" | "YES" | "true" | "TRUE"))
        .unwrap_or(false)
}

fn has_rustflag_token(token: &str) -> bool {
    let encoded = std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
    encoded.split('\u{1f}').any(|f| f == token)
}

fn has_cfg_axtest() -> bool {
    if has_rustflag_token("--cfg=axtest") {
        return true;
    }
    let encoded = std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
    let mut prev_cfg = false;
    for flag in encoded.split('\u{1f}') {
        if prev_cfg && flag == "axtest" {
            return true;
        }
        prev_cfg = flag == "--cfg";
    }
    false
}

fn has_instrument_coverage() -> bool {
    let encoded = std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
    encoded
        .split('\u{1f}')
        .any(|f| f == "instrument-coverage" || f == "-Cinstrument-coverage")
}

fn main() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let platform = axconfig::PLATFORM;
    if platform != "dummy" {
        gen_linker_script(&arch, platform).unwrap();
    }
}

fn gen_linker_script(arch: &str, platform: &str) -> Result<()> {
    let fname = format!("linker_{platform}.lds");
    let output_arch = if arch == "x86_64" {
        "i386:x86-64"
    } else if arch.contains("riscv") {
        "riscv" // OUTPUT_ARCH of both riscv32/riscv64 is "riscv"
    } else {
        arch
    };
    let ld_content = std::fs::read_to_string("linker.lds.S")?;
    let ld_content = ld_content.replace("%ARCH%", output_arch);
    let ld_content = ld_content.replace(
        "%KERNEL_BASE%",
        &format!("{:#x}", axconfig::plat::KERNEL_BASE_VADDR),
    );
    let ld_content = ld_content.replace("%CPU_NUM%", &format!("{}", axconfig::plat::MAX_CPU_NUM));
    let ax_test_enabled = env_flag("AX_TEST") || has_cfg_axtest();
    let ax_test_cov_enabled = env_flag("AX_TEST_COV") || has_instrument_coverage();
    let ld_content = ld_content.replace(
        "%AXTEST_SECTIONS%",
        if ax_test_enabled { AXTEST_SECTIONS } else { "" },
    );
    let ld_content = ld_content.replace(
        "%LLVM_PROFILE_SECTIONS%",
        if ax_test_cov_enabled {
            LLVM_PROFILE_SECTIONS
        } else {
            ""
        },
    );

    // target/<target_triple>/<mode>/build/axhal-xxxx/out
    let out_dir = std::env::var("OUT_DIR").unwrap();
    // target/<target_triple>/<mode>/linker_xxxx.lds
    let out_path = Path::new(&out_dir).join("../../..").join(fname);
    std::fs::write(out_path, ld_content)?;
    Ok(())
}
