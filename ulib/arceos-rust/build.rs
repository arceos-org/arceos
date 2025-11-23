// Package name of the `lib` directory
const LIB_NAME: &str = "arceos_rust_interface";

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!(
        "cargo:warning=Running build script for ArceOS rust library. Time: {:?}",
        std::time::SystemTime::now()
    );
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_dir = manifest_dir.join("lib");

    // generate configuration file
    let config_path = generate_config(&manifest_dir, &out_dir);
    println!("cargo:warning=config path: {}", config_path.display());
    // build the ArceOS library
    let artifact_path = compile_foo_project(&lib_dir, &out_dir, &config_path);
    // rename symbols to avoid conflicts
    let rename_list = lib_dir.join("symbol_rename.txt");
    let lib_file = artifact_path.join(format!("lib{}.a", LIB_NAME));
    rename_symbols(&lib_file, &rename_list);
    // copy linker script
    let linker_script_path = artifact_path.join(format!("linker_{}.lds", get_platform()));
    let out_linker_script_path = env::var("AX_LINK")
        .map(PathBuf::from)
        .unwrap_or(out_dir)
        .join("link.lds");
    std::fs::copy(&linker_script_path, &out_linker_script_path)
        .expect("Failed to copy linker script.");
    println!(
        "cargo:warning=Linker script path: {}",
        out_linker_script_path.display()
    );

    println!("cargo:rustc-link-search=native={}", artifact_path.display());
    println!("cargo:rustc-link-lib=static={}", LIB_NAME);

    // Trick: specify a non-existent path to always trigger a rebuild
    // See https://doc.rust-lang.org/cargo/faq.html#why-is-cargo-rebuilding-my-code
    println!("cargo:rerun-if-changed=always");
}

fn generate_config(manifest_dir: &PathBuf, out_dir: &PathBuf) -> PathBuf {
    let template = manifest_dir.join("defconfig.toml");
    let arch = get_arch();
    let platform = get_platform();

    // get platform config path
    let output = Command::new(cargo())
        .arg("axplat")
        .arg("info")
        .arg(format!("axplat-{}", platform))
        .arg("-c")
        .output()
        .expect("Failed to get platform config path.");

    if !output.status.success() {
        panic!("Failed to get platform config path.");
    }

    let platform_config_path = String::from_utf8_lossy(&output.stdout);
    let out_config_path = out_dir.join("axconfig.toml");

    let command = Command::new("axconfig-gen")
        .arg(&template)
        .arg(&platform_config_path.trim())
        .arg("-w")
        .arg(format!(r#"arch="{}""#, &arch))
        .arg("-w")
        .arg(format!(r#"platform="{}""#, get_platform()))
        .arg("-o")
        .arg(&out_config_path)
        .status()
        .expect("Failed to generate configuration file.");

    if !command.success() {
        panic!("Failed to generate configuration file.");
    }

    out_config_path
}

fn compile_foo_project(lib_dir: &PathBuf, out_dir: &PathBuf, config_path: &PathBuf) -> PathBuf {
    let profile = env::var("PROFILE").unwrap();
    let is_debug = profile == "debug";
    let arch = get_arch();
    let target = get_target(&arch);
    let features = env::var("CARGO_CFG_FEATURE").unwrap();
    let feature_list = features.replace(",", " ");

    let mut command = Command::new(cargo());
    command.env("AX_TARGET", &target);
    command.env("AX_MODE", profile);
    command.env("AX_CONFIG_PATH", config_path);
    if env::var("AX_IP").is_err() {
        command.env("AX_IP", "10.0.2.15");
    }
    if env::var("AX_GW").is_err() {
        command.env("AX_GW", "10.0.2.2");
    }
    command
        .current_dir(lib_dir)
        .arg("build")
        .arg("--target-dir")
        .arg(out_dir)
        .arg("--target")
        .arg(target)
        .arg("--no-default-features");
    if !feature_list.is_empty() {
        command.arg("--features").arg(feature_list);
    }
    if !is_debug {
        command.arg("--release");
    }
    println!(
        "cargo:warning=FATURES are {}",
        env::var("CARGO_CFG_FEATURE").unwrap_or("none".to_string())
    );
    println!("cargo:warning=command: {:?}", command);

    let status = command.status().expect("Failed to build ArceOS library.");

    if !status.success() {
        panic!("Failed to build ArceOS library.");
    }

    let build_type = if is_debug { "debug" } else { "release" };
    let lib_path = out_dir.join(&target).join(build_type);

    lib_path
}

fn cargo() -> String {
    env::var("CARGO").unwrap()
}

fn rename_symbols(lib_path: &Path, rename_list: &Path) {
    Command::new("objcopy")
        .arg("--redefine-syms")
        .arg(rename_list)
        .arg(lib_path)
        .status()
        .expect("Failed to rename symbols in the library.");
}

fn get_arch() -> String {
    env::var("CARGO_CFG_TARGET_ARCH").unwrap()
}

fn get_target(arch: &str) -> &'static str {
    match arch {
        "x86_64" => "x86_64-unknown-none",
        "aarch64" => "aarch64-unknown-none-softfloat",
        "riscv64" => "riscv64gc-unknown-none-elf",
        "loongarch64" => "loongarch64-unknown-none-softfloat",
        _ => panic!("Unsupported architecture: {}", arch),
    }
}

fn get_platform() -> &'static str {
    let arch = get_arch();
    match arch.as_ref() {
        "x86_64" => "x86-pc",
        "aarch64" => "aarch64-qemu-virt",
        "riscv64" => "riscv64-qemu-virt",
        "loongarch64" => "loongarch64-qemu-virt",
        _ => panic!("Unsupported architecture: {}", arch),
    }
}
