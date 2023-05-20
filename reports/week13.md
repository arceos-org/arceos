# 第十三周汇报

**致理-信计01  佟海轩 2020012709**

## 本周进展

### 适配 axdriver 接口更改

学习 `smoltcp_impl` 的适配方式，进行适配。

修改后出现问题：

``` log
[  0.756386 0 axruntime::lang_items:5] panicked at 'misaligned pointer dereference: address must be a multiple of 0x8 but is 0xffffffc0802cddf4', ulib/libax/src/cbindings/malloc.rs:30:42
```

gdb 调试后发现，`Slab` 在 pop `free_block_list` 时 pop 出了未对齐的块。🤔

或许是之前回收块时 push 进了未对齐的块？

打 log 发现：

``` log
[  0.747836 0 slab_allocator::slab:59] deallocating 64 Bytes block: 0xffffffc0802cddf4
```

地址 `0xffffffc0802cddf4` 未被分配过，但被回收。

gdb 调试发现，这个问题在 lwip_impl 中一处 log 中出现：

``` rust
info!(
    "DNS found: name={} ipaddr={}",
    CString::from_raw(name as *mut c_char).to_str().unwrap(),
    IpAddr::from(*ipaddr)
);
```

发现是 `CString::from_raw` 使用出错，该函数认为会获取指针指向数据的所有权，在使用后会负责进行回收，于是导致重复回收。

此处应使用 `CStr::from_ptr`：

``` rust
info!(
    "DNS found: name={} ipaddr={}",
    CStr::from_ptr(name as *mut c_char).to_str().unwrap(),
    IpAddr::from(*ipaddr)
);
```

修复后，C 和 Rust 应用均可正常运行。

### x86_64 / aarch64 适配

编译对应 QEMU，安装对应工具链。

在 `build.rs` 中通过 `let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();` 获取目标架构，进行对应配置。

x86_64 未遇到问题。

aarch64 编译失败：

``` log
error: linking with `rust-lld` failed: exit status: 1
  |
  = note: LC_ALL="C" PATH="/home/thx/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin:/home/thx/.local/bin:/home/thx/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/s"
  = note: rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(err.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(init.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(def.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(dns.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(inet_chksum.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(ip.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(mem.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(memp.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(netif.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(pbuf.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(raw.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(stats.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(sys.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(altcp.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(altcp_alloc.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(altcp_tcp.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(tcp.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(tcp_in.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(tcp_out.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: /home/thx/Git/arceos-lwip/target/aarch64-unknown-none-softfloat/debug/deps/liblwip_rust-6bbbb339c252586b.rlib(timeouts.o) is incompatible with /tmp/rustcXDH4TM/symbols.o
          rust-lld: error: too many errors emitted, stopping now (use --error-limit=0 to see all errors)


error: could not compile `arceos-httpclient` (bin "arceos-httpclient") due to previous error
```

发现均是与 `symbols.o` 不兼容。

考虑到之前编译 riscv64 版本时，未额外配置时会有 `cannot link object files with different floating-point ABI from /tmp/rustcjJ6QUD/symbols.o`，怀疑也是浮点 ABI 导致的问题。

尝试增加 `-mfloat-abi=???(soft/softfp/hard)` 的参数后提示未知的参数。

尝试分析 `build.rs` 的 log 发现，似乎并没有使用正确的编译器。

指定编译器后解决问题。

``` rust
match arch {
    "riscv64" => {
        base_config.compiler("riscv64-linux-musl-gcc");
        base_config.flag("-mabi=lp64d");
    }
    "aarch64" => {
        base_config.compiler("aarch64-linux-musl-gcc");
    }
    "x86_64" => {
        base_config.compiler("x86_64-linux-musl-gcc");
    }
    _ => {
        panic!("Unsupported arch: {}", arch);
    }
}
```

### CI

之前由于没有多架构的适配，把 CI 临时去除了，现在加回来。

出现较多问题，仍在解决中。

例如：`make A=apps/net/httpclient LOG=info NET=y ARCH=x86_64 ACCEL=n` 出错，riscv64 和 aarch64 无问题。

``` log
error: linking with `rust-lld` failed: exit status: 1
  |
  = note: LC_ALL="C" PATH="/home/thx/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin:/home/thx/.local/bin:/home/thx/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin:/home/thx/src/qemu-8.0.0/build:/home/thx/src/riscv64-linux-musl-cross/bin:/home/thx/src/x86_64-linux-musl-cross/bin:/home/thx/src/aarch64-linux-musl-cross/bin" VSLANG="1033" "rust-lld" "-flavor" "gnu" "/tmp/rustc5WWt0d/symbols.o" "/home/thx/Git/arceos-lwip/target/x86_64-unknown-none/release/deps/arceos_httpclient-81d249547e86c1dc.arceos_httpclient.27eb1be0a13a6b9d-cgu.9.rcgu.o" "--as-needed" "-L" "/home/thx/Git/arceos-lwip/target/x86_64-unknown-none/release/deps" "-L" "/home/thx/Git/arceos-lwip/target/release/deps" "-L" "/home/thx/Git/arceos-lwip/target/x86_64-unknown-none/release/build/lwip_rust-028903128b5a9c5d/out" "-L" "/home/thx/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-none/lib" "-Bstatic" "/tmp/rustc5WWt0d/liblwip_rust-f01b4fd7c2e53b1f.rlib" "/home/thx/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-none/lib/libcompiler_builtins-55df7e66f8cf4952.rlib" "-Bdynamic" "--eh-frame-hdr" "-z" "noexecstack" "-L" "/home/thx/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-none/lib" "-o" "/home/thx/Git/arceos-lwip/target/x86_64-unknown-none/release/deps/arceos_httpclient-81d249547e86c1dc" "--gc-sections" "-pie" "-z" "relro" "-z" "now" "-O1" "-T/home/thx/Git/arceos-lwip/modules/axhal/linker_x86_64.lds" "-no-pie"
  = note: rust-lld: error: /tmp/rustc5WWt0d/liblwip_rust-f01b4fd7c2e53b1f.rlib(nd6.o):(function nd6_send_rs: .text.nd6_send_rs+0xaf): relocation R_X86_64_REX_GOTPCRELX out of range: -549753436244 is not in [-2147483648, 2147483647]; references ip6_addr_any
          >>> referenced by nd6.c
          >>> defined in /tmp/rustc5WWt0d/liblwip_rust-f01b4fd7c2e53b1f.rlib(ip6_addr.o)

          rust-lld: error: /tmp/rustc5WWt0d/liblwip_rust-f01b4fd7c2e53b1f.rlib(nd6.o):(function nd6_send_ns: .text.nd6_send_ns+0x143): relocation R_X86_64_REX_GOTPCRELX out of range: -549753436244 is not in [-2147483648, 2147483647]; references ip6_addr_any
          >>> referenced by nd6.c
          >>> defined in /tmp/rustc5WWt0d/liblwip_rust-f01b4fd7c2e53b1f.rlib(ip6_addr.o)

          rust-lld: error: /tmp/rustc5WWt0d/liblwip_rust-f01b4fd7c2e53b1f.rlib(nd6.o):(function nd6_find_route: .text.nd6_find_route+0x8a): relocation R_X86_64_REX_GOTPCRELX out of range: -549749819748 is not in [-2147483648, 2147483647]; references default_router_list
          >>> referenced by nd6.c
          >>> defined in /tmp/rustc5WWt0d/liblwip_rust-f01b4fd7c2e53b1f.rlib(nd6.o)

          rust-lld: error: /tmp/rustc5WWt0d/liblwip_rust-f01b4fd7c2e53b1f.rlib(nd6.o):(function nd6_get_next_hop_addr_or_queue: .text.nd6_get_next_hop_addr_or_queue+0x2bf): relocation R_X86_64_REX_GOTPCRELX out of range: -549749819748 is not in [-2147483648, 2147483647]; references default_router_list
          >>> referenced by nd6.c
          >>> defined in /tmp/rustc5WWt0d/liblwip_rust-f01b4fd7c2e53b1f.rlib(nd6.o)


error: could not compile `arceos-httpclient` (bin "arceos-httpclient") due to previous error
make: *** [scripts/make/build.mk:16: _cargo_build] Error 101
```

## 下周计划

修复 CI，优化代码，进行 PR
