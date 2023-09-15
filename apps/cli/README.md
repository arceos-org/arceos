
### How to test cli app
```

# make A=apps/cli ARCH=aarch64 LOG=debug
    Building App: cli, Arch: aarch64, Platform: aarch64-qemu-virt, App type: rust
cargo build --target aarch64-unknown-none-softfloat --target-dir /root/Github/Chenlong/arceos/target --release  --manifest-path apps/cli/Cargo.toml --features "axstd/log-level-debug"
   Compiling axlog v0.1.0 (/root/Github/Chenlong/arceos/modules/axlog)
   Compiling axio v0.1.0 (/root/Github/Chenlong/arceos/crates/axio)
   Compiling axhal v0.1.0 (/root/Github/Chenlong/arceos/modules/axhal)
   Compiling axruntime v0.1.0 (/root/Github/Chenlong/arceos/modules/axruntime)
   Compiling axfeat v0.1.0 (/root/Github/Chenlong/arceos/api/axfeat)
   Compiling arceos_api v0.1.0 (/root/Github/Chenlong/arceos/api/arceos_api)
   Compiling axstd v0.1.0 (/root/Github/Chenlong/arceos/ulib/axstd)
   Compiling arceos-cli v0.1.0 (/root/Github/Chenlong/arceos/apps/cli)
    Finished release [optimized] target(s) in 1.56s
rust-objcopy --binary-architecture=aarch64 apps/cli/cli_aarch64-qemu-virt.elf --strip-all -O binary apps/cli/cli_aarch64-qemu-virt.bin

# make A=apps/cli ARCH=aarch64 LOG=debug run
    Building App: cli, Arch: aarch64, Platform: aarch64-qemu-virt, App type: rust
cargo build --target aarch64-unknown-none-softfloat --target-dir /root/Github/Chenlong/arceos/target --release  --manifest-path apps/cli/Cargo.toml --features "axstd/log-level-debug"
    Finished release [optimized] target(s) in 0.09s
rust-objcopy --binary-architecture=aarch64 apps/cli/cli_aarch64-qemu-virt.elf --strip-all -O binary apps/cli/cli_aarch64-qemu-virt.bin
    Running on qemu...
qemu-system-aarch64 -m 128M -smp 1 -cpu cortex-a72 -machine virt -kernel apps/cli/cli_aarch64-qemu-virt.bin -nographic

       d8888                            .d88888b.   .d8888b.
      d88888                           d88P" "Y88b d88P  Y88b
     d88P888                           888     888 Y88b.
    d88P 888 888d888  .d8888b  .d88b.  888     888  "Y888b.
   d88P  888 888P"   d88P"    d8P  Y8b 888     888     "Y88b.
  d88P   888 888     888      88888888 888     888       "888
 d8888888888 888     Y88b.    Y8b.     Y88b. .d88P Y88b  d88P
d88P     888 888      "Y8888P  "Y8888   "Y88888P"   "Y8888P"

arch = aarch64
platform = aarch64-qemu-virt
target = aarch64-unknown-none-softfloat
smp = 1
build_mode = release
log_level = debug

[  0.004989 0 axruntime:126] Logging is enabled.
[  0.006520 0 axruntime:127] Primary CPU 0 started, dtb = 0x44000000.
[  0.007244 0 axruntime:129] Found physcial memory regions:
[  0.007958 0 axruntime:131]   [PA:0x40080000, PA:0x40089000) .text (READ | EXECUTE | RESERVED)
[  0.009000 0 axruntime:131]   [PA:0x40089000, PA:0x4008c000) .rodata (READ | RESERVED)
[  0.009624 0 axruntime:131]   [PA:0x4008c000, PA:0x40090000) .data .tdata .tbss .percpu (READ | WRITE | RESERVED)
[  0.010322 0 axruntime:131]   [PA:0x40090000, PA:0x400d0000) boot stack (READ | WRITE | RESERVED)
[  0.011015 0 axruntime:131]   [PA:0x400d0000, PA:0x400d1000) .bss (READ | WRITE | RESERVED)
[  0.011629 0 axruntime:131]   [PA:0x400d1000, PA:0x48000000) free memory (READ | WRITE | FREE)
[  0.012414 0 axruntime:131]   [PA:0x9000000, PA:0x9001000) mmio (READ | WRITE | DEVICE | RESERVED)
[  0.013030 0 axruntime:131]   [PA:0x8000000, PA:0x8020000) mmio (READ | WRITE | DEVICE | RESERVED)
[  0.013626 0 axruntime:131]   [PA:0xa000000, PA:0xa004000) mmio (READ | WRITE | DEVICE | RESERVED)
[  0.014252 0 axruntime:131]   [PA:0x10000000, PA:0x3eff0000) mmio (READ | WRITE | DEVICE | RESERVED)
[  0.014855 0 axruntime:131]   [PA:0x4010000000, PA:0x4020000000) mmio (READ | WRITE | DEVICE | RESERVED)
[  0.015583 0 axruntime:149] Initialize platform devices...
[  0.016038 0 axruntime:185] Primary CPU 0 init OK.
Available commands:
  exit
  help
  uname
  ldr
  str
arceos# help
Available commands:
  exit
  help
  uname
  ldr
  str
arceos# uname
ArceOS 0.1.0 aarch64 aarch64-qemu-virt
arceos# ldr 
ldr
try: ldr ffff0000400fe000 / ldr ffff000040080000 ffff000040080008
arceos# str
str
try: str ffff0000400fe000 12345678
arceos# str ffff000040080000 11223344
str
First element: ffff000040080000
Second element: 11223344
addr = ffff000040080000
val = 11223344
Parsed address: 0xffff000040080000
Parsed value: 0x11223344
Write value at address ffff000040080000: 0x11223344
arceos# ldr ffff000040080000
ldr
addr = ffff000040080000
Parsed address: 0xffff000040080000
Value at address ffff000040080000: 0x11223344
arceos# exit
Bye~
[ 46.110566 0 axhal::platform::aarch64_common::psci:96] Shutting down...
```
