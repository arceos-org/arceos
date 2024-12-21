# ArceOS Build Flow

We will provide an example to illustrate the process of building and running ArceOS:

**Examples:**

What happens when "make A=apps/net/httpserver ARCH=aarch64 LOG=info NET=y SMP=1 run" is executed?

- How ArceOS build?
    - Firstly check Makefile: Based on different parameters, select whether FS/NET/GRAPHIC param is yes or not. If it is y, it will be compiled in conditional compilation.
    - `cargo.mk` determines whether to add the corresponding feature based on whether FS/NET/GRAPHIC is set to y.
    ```
    features-$(FS) += libax/fs
    features-$(NET) += libax/net
    features-$(GRAPHIC) += libax/display
    ```

    - `_cargo_build`: The `_cargo_build` method is defined in cargo.mk. Different compilation methods are selected based on the language. For example, for Rust, when `cargo_build,--manifest-path $(APP)/Cargo.toml` is called, where $(APP) represents the current application to be run.
    - Taking httpserver as an example, let's see how ArceOS are conditionally compiled. First, in the `Cargo.toml` file of httpserver, the dependency is specified as: `libax = { path = "../../../ulib/libax", features = ["paging", "multitask", "net"] }`. This indicates that libax needs to be compiled and has the three features mentioned above.
    - After checking libax, the following three features were found:
        - `paging = ["axruntime/paging"]`
        - `multitask = ["axruntime/multitask", "axtask/multitask", "axsync/multitask"]`
        - `net = ["axruntime/net", "dep:axnet"]`

        This involves modules such as axruntime, axtask, axsync, etc., and conditional compilation is performed on these modules.
    - The above are some modules required for compilation, next we will look at how to perform conditional compilation. The `cargo.mk` file describes how to use the cargo method for conditional compilation, with the following build parameters:
    ```
    build_args := \
    -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem \
    --config "build.rustflags='-Clink-arg=-T$(LD_SCRIPT)'" \
    --target $(TARGET) \
    --target-dir $(CURDIR)/target \
    --features "$(features-y)" \
    ```
    Note that the -Zbuild-std option is mentioned here, indicating the replacement of the standard library for the application and the use of libraries provided by ArceOS.

    - Therefore, to summarize: choose conditions in Makefile and select the corresponding app directory for conditional compilation in `cargo.mk`.
- Next, describe how ArceOS run:
    - Firstly, examining the Makefile reveals that in addition to building, running an application also requires `justrun`.
    - Following this, it was found that the `qemu.mk` file would call run_qemu. Similar to the build process, the execution process would also use conditional selection and run.
    - At runtime, Arceos first performs some boot operations, such as executing in the riscv64 environment:
    ```rust
    #[naked]
    #[unsafe(no_mangle)]
    #[unsafe(link_section = ".text.boot")]
    unsafe extern "C" fn _start() -> ! {
        unsafe extern "C" {
            fn rust_main();
        }
        // PC = 0x8020_0000
        // a0 = hartid
        // a1 = dtb
        core::arch::naked_asm!("
            mv      s0, a0                  // save hartid
            mv      s1, a1                  // save DTB pointer
            la      sp, {boot_stack}
            li      t0, {boot_stack_size}
            add     sp, sp, t0              // setup boot stack

            call    {init_boot_page_table}
            call    {init_mmu}              // setup boot page table and enabel MMU

            li      s2, {phys_virt_offset}  // fix up virtual high address
            add     sp, sp, s2

            mv      a0, s0
            mv      a1, s1
            la      a2, {platform_init}
            add     a2, a2, s2
            jalr    a2                      // call platform_init(hartid, dtb)

            mv      a0, s0
            mv      a1, s1
            la      a2, {rust_main}
            add     a2, a2, s2
            jalr    a2                      // call rust_main(hartid, dtb)
            j       .",
            phys_virt_offset = const PHYS_VIRT_OFFSET,
            boot_stack_size = const TASK_STACK_SIZE,
            boot_stack = sym BOOT_STACK,
            init_boot_page_table = sym init_boot_page_table,
            init_mmu = sym init_mmu,
            platform_init = sym super::platform_init,
            rust_main = sym rust_main,
        )
    }
    ```
    - Later, it jumps to `rust_main` in `axruntime` to run. After some conditional initialization, `rust_main` executes `main()`. Since this main is defined by the application, symbol linkage should be established and jumped to (no context switch is needed since it's a single address space).

    -  Then, the user program begins executing through `libax`'s API. The application runs in kernel mode, without the need for syscall and context switching, resulting in higher efficiency.
