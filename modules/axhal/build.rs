use std::{io::Result, path::PathBuf};

const LINKER_SCRIPT_NAME: &str = "linker.x";

fn main() {
    println!("cargo:rustc-check-cfg=cfg(plat_dyn)");

    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let has_plat_dyn = std::env::var_os("CARGO_FEATURE_PLAT_DYN").is_some();
    let platform = axconfig::PLATFORM;

    if has_plat_dyn && target_os == "none" {
        println!("cargo:rustc-cfg=plat_dyn");
    }

    if platform != "dummy" {
        gen_linker_script(&arch, platform).unwrap();
    }
}

fn gen_linker_script(arch: &str, platform: &str) -> Result<()> {
    let legacy_fname = format!("linker_{platform}.lds");
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
    let ld_content = ld_content.replace(
        "%DWARF%",
        if std::env::var("DWARF").is_ok_and(|v| v == "y") {
            r#"debug_abbrev : { . += SIZEOF(.debug_abbrev); }
    debug_addr : { . += SIZEOF(.debug_addr); }
    debug_aranges : { . += SIZEOF(.debug_aranges); }
    debug_info : { . += SIZEOF(.debug_info); }
    debug_line : { . += SIZEOF(.debug_line); }
    debug_line_str : { . += SIZEOF(.debug_line_str); }
    debug_ranges : { . += SIZEOF(.debug_ranges); }
    debug_rnglists : { . += SIZEOF(.debug_rnglists); }
    debug_str : { . += SIZEOF(.debug_str); }
    debug_str_offsets : { . += SIZEOF(.debug_str_offsets); }"#
        } else {
            ""
        },
    );

    // target/<target_triple>/<mode>/build/axhal-xxxx/out
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-arg=-T{LINKER_SCRIPT_NAME}");

    // target/<target_triple>/<mode>/build/axhal-xxxx/out/linker.x
    std::fs::write(out_dir.join(LINKER_SCRIPT_NAME), &ld_content)?;

    // Keep a stable copy under target/<target_triple>/<mode>/ for callers
    // that still link outside Cargo build-script search paths.
    let target_dir = out_dir.join("../../..");
    std::fs::write(target_dir.join(LINKER_SCRIPT_NAME), &ld_content)?;
    std::fs::write(target_dir.join(legacy_fname), ld_content)?;
    Ok(())
}
