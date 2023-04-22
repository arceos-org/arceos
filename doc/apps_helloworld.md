# INTRODUCTION
| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [helloworld](../apps/helloworld/) | | | A minimal app that just prints a string |

# RUN

```shell
make A=apps/helloworld SMP=4 LOG=debug run
```

# STEPS

## step1
[init](./init.md)

After executed all initial actions, then arceos calls `main` function in `helloworld` app.

## step2

```Rust
fn main() {
    libax::println!("Hello, world!");
}
```

**flow chart**

```mermaid
graph TD;
    A[main] --> B["libax::println!(Hello, world!)"];
    B --> C[libax:io::__print_impl];
    C --> D[INLINE_LOCK=Mutex::new];
    C --> _guard=INLINE_LOCK.lock;
    C --> E["stdout().write_fmt(args)"];
```

### step2.1

```mermaid
graph TD;
    T["stdout()"] --> A["libax::io::stdio.rs::stdout()"];
    A --> B["INSTANCE: Mutex<StdoutRaw> = Mutex::new(StdoutRaw)"];
    A --> C["return Stdout { inner: &INSTANCE }"];
```

### step2.2

```mermaid
graph TD;
    T["stdout().write_fmt(args)"] --> A["Stdout::write"];
    A --> B["self.inner.lock().write(buf)"];
    B --> C["StdoutRaw::write"];
    C --> D["axhal::console::write_bytes(buf);"];
    C --> E["Ok(buf.len())"];
    D --> F["putchar"];
    F --> G["axhal::platform::qemu_virt_riscv::console::putchar"];
    G --> H["sbi_rt::legacy::console_putchar"];
```
