// Package name of the `lib` directory
const LIB_NAME: &str = "arceos_rust_interface";

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

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

    // Also copy to artifact_path so it's in the linker search path
    let artifact_linker_script = artifact_path.join("link.lds");
    std::fs::copy(&linker_script_path, &artifact_linker_script)
        .expect("Failed to copy linker script to artifact path.");

    println!(
        "cargo:warning=Linker script path: {}",
        artifact_linker_script.display()
    );

    println!("cargo:rustc-link-search=native={}", artifact_path.display());
    println!("cargo:rustc-link-lib=static={}", LIB_NAME);

    // Trick: specify a non-existent path to always trigger a rebuild
    // See https://doc.rust-lang.org/cargo/faq.html#why-is-cargo-rebuilding-my-code
    println!("cargo:rerun-if-changed=always");
}

fn generate_config(manifest_dir: &Path, out_dir: &Path) -> PathBuf {
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
        .arg(platform_config_path.trim())
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
    let mut feature_list = features.replace(",", " ");

    // Add myplat feature if custom platform is specified
    if let Ok(platform) = env::var("AX_PLATFORM") {
        feature_list.push_str(" myplat axplat-");
        feature_list.push_str(platform.as_str());
    } else {
        feature_list.push_str(" defplat");
    }

    println!("cargo:warning=FEATURES are {}", feature_list);

    let mut command = Command::new(cargo());
    command.env("AX_TARGET", target);
    command.env("AX_MODE", profile);
    command.env("AX_CONFIG_PATH", config_path);
    command.env("AX_LOG", get_log_level(&features));
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
    println!("cargo:warning=command: {:?}", command);

    let status = command.status().expect("Failed to build ArceOS library.");

    if !status.success() {
        panic!("Failed to build ArceOS library.");
    }

    let build_type = if is_debug { "debug" } else { "release" };
    out_dir.join(target).join(build_type)
}

fn cargo() -> String {
    env::var("CARGO").unwrap()
}

fn rename_symbols(lib_path: &Path, rename_list: &Path) {
    let output = Command::new("rust-objcopy")
        .arg("--redefine-syms")
        .arg(rename_list)
        .arg(lib_path)
        .output();

    match output {
        Ok(output) if output.status.success() => {}
        Ok(output) => panic!(
            "Failed to rename symbols with rust-objcopy (exit: {}).\nstdout:\n{}\nstderr:\n{}",
            output
                .status
                .code()
                .map_or_else(|| "signal".to_string(), |c| c.to_string()),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ),
        Err(_) => panic!(
            "Failed to run rust-objcopy. Please install required tools with:\n  rustup component \
             add llvm-tools-preview\n  cargo install cargo-binutils"
        ),
    }
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

fn get_platform() -> String {
    // Check if custom platform is specified via environment variable
    if let Ok(custom_platform) = env::var("AX_PLATFORM") {
        return custom_platform;
    }

    // Default platform based on architecture
    let arch = get_arch();
    match arch.as_ref() {
        "x86_64" => "x86-pc",
        "aarch64" => "aarch64-qemu-virt",
        "riscv64" => "riscv64-qemu-virt",
        "loongarch64" => "loongarch64-qemu-virt",
        _ => panic!("Unsupported architecture: {}", arch),
    }
    .to_string()
}

fn get_log_level(feature_list: &str) -> &str {
    let mut level = "off";
    for feature in feature_list.split(',') {
        if let Some(stripped) = feature.strip_prefix("log-level-") {
            level = stripped;
        }
    }
    level
}
