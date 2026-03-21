# ArceOS Rust std support

`arceos-rust` crate provides support for running rust app with Rust standard library on bare metal.

It includes implementations of various standard library features, such as memory management, threading, and I/O.

Examples are under `examples/std` directory.

## Get started

To use `arceos-rust` in your project, add `arceos-rust` to your `Cargo.toml`, such as:

```toml
[target.'cfg(target_os = "hermit")'.dependencies]
arceos-rust = { workspace = true, default-features = true, features = ["multitask", "log-level-off", "irq"] }
```

Then add the following to your `main.rs`:

```rust
#[cfg(target_os = "hermit")]
use arceos_rust as _;
```

Now, your Rust application can be run either on normal OS or on bare metal with ArceOS. You can use standard library features, such as `std::thread` and `std::fs`, without worrying about the underlying platform.

## Build

You can build your Rust application with `cargo build` as usual, run this command under your project folder:

```bash
RUSTFLAGS="-C link-arg=-no-pie -C link-arg=-T$(pwd)/link.lds" AX_LINK=$(pwd) cargo build --target x86_64-unknown-hermit -Zbuild-std=std,panic_abort
```

## Run

You can run your Rust application on qemu bare metal:

```bash
qemu-system-x86_64 -m 128M -smp 1 -machine q35 -nographic -kernel <your-app>
```

If you want network support, you can add:

```bash
-device virtio-net-pci,netdev=net0 -netdev user,id=net0
```

If you want to mount a disk image, you can add:

```bash
-device virtio-blk-pci,drive=disk0 -drive id=disk0,if=none,format=raw,file=disk.img
```

Final command line is:

```bash
qemu-system-x86_64 -m 128M -smp 1 -machine q35 -nographic -device virtio-net-pci,netdev=net0 -netdev user,id=net0 -device virtio-blk-pci,drive=disk0 -drive id=disk0,if=none,format=raw,file=disk.img -kernel arce_agent
```

## Example: ArceAgent

ArceAgent is a Rust application running on ArceOS, which serves as an agent for LLM-based applications. It demonstrates the use of Rust standard library features on ArceOS, including threading, networking, and file I/O.

To build ArceAgent, run the following command in `examples/std/arce_agent`:

```bash
cd examples/std/arce_agent
RUSTFLAGS="-C link-arg=-no-pie -C link-arg=-T$(pwd)/link.lds" AX_LINK=$(pwd) cargo build --target x86_64-unknown-hermit -Zbuild-std=std,panic_abort
```

Then you need to set up a proxy server on the host machine to forward HTTP requests from ArceAgent to an OpenAI-compatible API endpoint, such as Tsinghua AI Platform. You can visit `https://lab.cs.tsinghua.edu.cn/ai-platform/` to get an API key and paste it into `llm_proxy.py`. You can also modify `llm_proxy.py` to use other OpenAI-compatible API endpoint. Run the proxy server with:

```bash
python3 llm_proxy.py
```

Finally, create a disk image and run ArceAgent on QEMU:

```bash
cd target/x86_64-unknown-hermit/debug
qemu-system-x86_64 -m 128M -smp 1 -machine q35 -nographic -device virtio-net-pci,netdev=net0 -netdev user,id=net0 -device virtio-blk-pci,drive=disk0 -drive id=disk0,if=none,format=raw,file=/path/to/disk.img -kernel *arce_agent
```
