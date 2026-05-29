use std::io::Result;
use std::path::Path;

fn main() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let platform = axconfig::PLATFORM;
    if platform != "dummy" {
        gen_linker_script(&arch, platform).unwrap();
    }
}

fn gen_linker_script(arch: &str, platform: &str) -> Result<()> {
    let fname = format!("linker_{platform}.lds");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_dir = Path::new(&manifest_dir).join("../..");
    let platform_ld = workspace_dir
        .join("configs")
        .join(format!("linker_{platform}.lds"));
    println!("cargo:rerun-if-changed=linker.lds.S");
    println!("cargo:rerun-if-changed={}", platform_ld.display());
    let ld_content = if platform_ld.exists() {
        std::fs::read_to_string(platform_ld)?
    } else {
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
        ld_content.replace("%CPU_NUM%", &format!("{}", axconfig::plat::MAX_CPU_NUM))
    };

    // target/<target_triple>/<mode>/build/axhal-xxxx/out
    let out_dir = std::env::var("OUT_DIR").unwrap();
    // target/<target_triple>/<mode>/linker_xxxx.lds
    let out_path = Path::new(&out_dir).join("../../..").join(fname);
    std::fs::write(out_path, ld_content)?;
    Ok(())
}
