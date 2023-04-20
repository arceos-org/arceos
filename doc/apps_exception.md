# INTRODUCTION

| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [exception](../apps/exception/) | | paging | Exception handling test |

# RUN

```console
$ make A=apps/exception LOG=debug run
...
Running exception tests...
[  0.249873 0 axhal::arch::riscv::trap:13] Exception(Breakpoint) @ 0xffffffc0802001e8
Exception tests run OK!
[  0.068358 0 axtask::api:6] main task exited: exit_code=0
[  0.069128 0 axhal::platform::qemu_virt_riscv::misc:2] Shutting down...
```

# STEPS

## step1

[init](./init.md)

After executed all initial actions, then arceos calls `main` function in `exception` app.

## step2

``` Rust
fn raise_break_exception() {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        asm!("int3");
        #[cfg(target_arch = "aarch64")]
        asm!("brk #0");
        #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
        asm!("ebreak");
    }
}

#[no_mangle]
fn main() {
    println!("Running exception tests...");
    raise_break_exception();
    println!("Exception tests run OK!");
}
```

**flow chart**

```mermaid
graph TD;
    A[" asm!(ebreak)"] --> B["raise exception"];
    B --> C["axhal::arch::riscv::trap.S::trap_vector_base"];
    C --> D["switch sscratch and sp"];
    C -- "from U mode" --> E["Ltrap_entry_u: SAVE_REGS 1; a1 <-- 1"];
    C -- "from S mode" --> F["Ltrap_entry_s: SAVE_REGS 0; a1 <-- 0"];
    E --> G[axhal::arch::riscv::trap::riscv_trap_handler];
    F --> G;
    G -- "Trap::Exception(E::Breakpoint)" --> H["handle_breakpoint(&mut tf.sepc)"];
    H --> I["debug!(Exception(Breakpoint) @ {:#x} , sepc);*sepc += 2;"];
    I -- "from U mode" --> J["Ltrap_entry_u: RESTORE_REGS 1"];
    I -- "from S mode" --> K["Ltrap_entry_s: RESTORE_REGS 0"];
    J --> L[sret];
    K --> L;
```
