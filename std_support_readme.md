# ArceOS Rust std support

`arceos-rust` lets Rust applications use `std` on bare metal (Hermit target) with ArceOS as runtime/kernel support.

Examples are under `examples/std`.

## Run the std examples

### Disk image preparation

Examples expect `disk.img` in `examples/std` directory. Both ext4 and FAT32 filesystems are supported. 

Recommended commands:

```bash
dd if=/dev/zero of=disk.img bs=1M count=64
# or truncate:
# truncate -s 64M disk.img
mkfs.ext4 -F disk.img
# or FAT32:
# mkfs.vfat -F 32 disk.img
```

### Run examples with cargo

Run from each example directory, for example:

```bash
cd examples/std/helloworld
cargo run --target riscv64gc-unknown-hermit
```

Supported targets used by examples:

- `x86_64-unknown-hermit`
- `riscv64gc-unknown-hermit`
- `aarch64-unknown-hermit`
- `loongarch64-unknown-hermit` uses a custom target JSON (`--target ../loongarch64-unknown-hermit.json -Zjson-target-spec`), see below

`examples/std/*/.cargo/config.toml` already provides:

- linker args (`-no-pie`, `-Tlink.lds`)
- per-arch QEMU runner
- `build-std` settings for build standard library

## Port an existing Rust project

### 1) Add dependency

```toml
[target.'cfg(target_os = "hermit")'.dependencies]
arceos-rust = { workspace = true, default-features = true, features = ["log-level-off"] }
```

NOTE: `default-features = true` must be specified, or the build script will not receive `default` at `CARGO_CFG_FEATURE`.

Then enable features based on your app:

- `fs` for file system
- `net` for networking
- `multitask`/`irq` if needed

For other available features, see document for `arceos-rust`.

### 2) Import runtime shim in `main.rs`

```rust
#[cfg(target_os = "hermit")]
use arceos_rust as _;
```

### 3) Add `.cargo/config.toml`

Use template:

```bash
mkdir -p .cargo
cp <ArceOS project dir>/examples/std/config.template.toml .cargo/config.toml
```

Then adjust runner args for your enabled features (network/disk), as described in template comments.

### 4) Build and run

```bash
cargo run --target <arch>-unknown-hermit
```

`<arch` can be `x86_64`, `riscv64gc`, `aarch64`, or `loongarch64` (with JSON target).

## Customize runner via `config.template.toml`

Template path:

- `examples/std/config.template.toml`

How to use:

1. Copy it to your project as `.cargo/config.toml`.
2. Keep the target section you need.
3. Edit the `runner` array to enable optional QEMU args.

Typical edits:

- Enable network:
  - `"-device", "virtio-net-pci,netdev=net0"`
  - `"-netdev", "user,id=net0"`
- Enable server port forward (example 5555):
  - `"-netdev", "user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555"`
- Enable disk:
  - `"-device", "virtio-blk-pci,drive=disk0"`
  - `"-drive", "id=disk0,if=none,format=raw,file=./disk.img"`

Default feature behavior is PCI bus, so template defaults to `virtio-*-pci`.

## LoongArch custom target

`loongarch64-unknown-hermit` is not a built-in Rust target. Use the JSON target spec shipped in `examples/std`.

Copy target spec to your project root (or reference it by path):

```bash
cp examples/std/loongarch64-unknown-hermit.json <your-project>/
```

Run with JSON target:

```bash
cargo run --target ./loongarch64-unknown-hermit.json -Zjson-target-spec
```

If you keep `target.loongarch64-unknown-hermit` in `.cargo/config.toml`, it applies only when target triple name matches exactly. For JSON-path target usage, use `target."cfg(...)"`/manual command line, or keep a dedicated local config for LoongArch.

## Custom platform support

1. Enable `bus-mmio` feature of `arceos-rust` if needed (depends on your platform).
2. Set environment variable `AX_PLATFORM` to your custom platform name when building.
3. Strip the artifact and make uboot image if needed. For example:

    ```bash
    rust-objcopy --strip-all -O binary <artifact> out.bin
    mkimage -A riscv -O linux -T kernel -C none -a 0x80200000 -d out.bin out.uimg
    ```
   
### Example: sg2002 platform support

1. Create a crate `axplat_riscv64_sg2002` and implement platform related code here.
2. Add platform related feature to crate `arceos-rust-interface`:
    
    ```toml
    [features]
    axplat-riscv64-sg2002 = ["dep:axplat-riscv64-sg2002"]
    
    [dependencies]
    axplat-riscv64-sg2002 = { workspace = true, optional = true }
    ```
   
3. Import platform crate if the feature is enabled:

    ```rust
    // lib.rs in `arceos-rust-interface`
    #[cfg(feature = "axplat-riscv64-sg2002")]
    extern crate axplat_riscv64_sg2002;
    ```

4. If user sets environment variable `AX_PLATFORM=riscv64-sg2002`, build script will automatically add feature `axplat-riscv64-sg2002`.
5. This platform use mmio, so user should enable feature `bus-mmio`.