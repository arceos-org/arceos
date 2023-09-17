
### 编译生成 img 文件的命令
```
make A=apps/boards/raspi4 ARCH=aarch64 PLATFORM=aarch64-raspi4 LOG=debug SMP=4 run
cat ../rust-raspberrypi-OS-tutorials/06_uart_chainloader/kernel8.img apps/boards/raspi4/raspi4_aarch64-raspi4.bin > kernel8.img
```

### 编译过程记录
```
# make A=apps/boards/raspi4 ARCH=aarch64 PLATFORM=aarch64-raspi4 LOG=debug SMP=4 run
    Building App: raspi4, Arch: aarch64, Platform: aarch64-raspi4, App type: rust
cargo build --target aarch64-unknown-none-softfloat --target-dir /root/Github/Chenlong/arceos/target --release  --manifest-path apps/boards/raspi4/Cargo.toml --features "axstd/log-level-debug axstd/smp"
   Compiling axconfig v0.1.0 (/root/Github/Chenlong/arceos/modules/axconfig)
   Compiling spinlock v0.1.0 (/root/Github/Chenlong/arceos/crates/spinlock)
   Compiling axhal v0.1.0 (/root/Github/Chenlong/arceos/modules/axhal)
   Compiling axlog v0.1.0 (/root/Github/Chenlong/arceos/modules/axlog)
   Compiling axruntime v0.1.0 (/root/Github/Chenlong/arceos/modules/axruntime)
   Compiling axfeat v0.1.0 (/root/Github/Chenlong/arceos/api/axfeat)
   Compiling arceos_api v0.1.0 (/root/Github/Chenlong/arceos/api/arceos_api)
   Compiling axstd v0.1.0 (/root/Github/Chenlong/arceos/ulib/axstd)
   Compiling arceos-raspi4 v0.1.0 (/root/Github/Chenlong/arceos/apps/boards/raspi4)
    Finished release [optimized] target(s) in 1.59s
rust-objcopy --binary-architecture=aarch64 apps/boards/raspi4/raspi4_aarch64-raspi4.elf --strip-all -O binary apps/boards/raspi4/raspi4_aarch64-raspi4.bin
    Running on qemu...
emu-system-aarch64 -m 2G -smp 4 -cpu cortex-a72 -machine raspi4b2g -kernel apps/boards/raspi4/raspi4_aarch64-raspi4.bin -nographic

       d8888                            .d88888b.   .d8888b.
      d88888                           d88P" "Y88b d88P  Y88b
     d88P888                           888     888 Y88b.
    d88P 888 888d888  .d8888b  .d88b.  888     888  "Y888b.
   d88P  888 888P"   d88P"    d8P  Y8b 888     888     "Y88b.
  d88P   888 888     888      88888888 888     888       "888
 d8888888888 888     Y88b.    Y8b.     Y88b. .d88P Y88b  d88P
d88P     888 888      "Y8888P  "Y8888   "Y88888P"   "Y8888P"

arch = aarch64
platform = aarch64-raspi4
target = aarch64-unknown-none-softfloat
smp = 4
build_mode = release
log_level = debug

[  0.013401 axruntime:126] Logging is enabled.
[  0.017938 axruntime:127] Primary CPU 0 started, dtb = 0x100.
[  0.020174 axruntime:129] Found physcial memory regions:
[  0.022343 axruntime:131]   [PA:0x80000, PA:0x87000) .text (READ | EXECUTE | RESERVED)
[  0.025306 axruntime:131]   [PA:0x87000, PA:0x89000) .rodata (READ | RESERVED)
[  0.027690 axruntime:131]   [PA:0x89000, PA:0x8d000) .data .tdata .tbss .percpu (READ | WRITE | RESERVED)
[  0.029109 axruntime:131]   [PA:0x8d000, PA:0x18d000) boot stack (READ | WRITE | RESERVED)
[  0.030433 axruntime:131]   [PA:0x18d000, PA:0x18e000) .bss (READ | WRITE | RESERVED)
[  0.031586 axruntime:131]   [PA:0x0, PA:0x1000) spintable (READ | WRITE | RESERVED)
[  0.032880 axruntime:131]   [PA:0x18e000, PA:0xfc000000) free memory (READ | WRITE | FREE)
[  0.034360 axruntime:131]   [PA:0xfe201000, PA:0xfe202000) mmio (READ | WRITE | DEVICE | RESERVED)
[  0.035648 axruntime:131]   [PA:0xff841000, PA:0xff849000) mmio (READ | WRITE | DEVICE | RESERVED)
[  0.036969 axruntime:149] Initialize platform devices...
[  0.038219 axruntime::mp:18] starting CPU 1...
[  0.040891 axruntime::mp:35] Secondary CPU 1 started.
[  0.045548 axruntime::mp:18] starting CPU 2...
[  0.046443 axruntime::mp:45] Secondary CPU 1 init OK.
[  0.055724 axruntime::mp:35] Secondary CPU 2 started.
[  0.056580 axruntime::mp:18] starting CPU 3...
[  0.058083 axruntime::mp:45] Secondary CPU 2 init OK.
[  0.063700 axruntime::mp:35] Secondary CPU 3 started.
[  0.064109 axruntime::mp:45] Secondary CPU 3 init OK.
[  0.068492 axruntime:185] Primary CPU 0 init OK.
Hello, world!
start
??`g1 
forward
??d~1 
2 
3 
4 
stop
??1 
turn left
??d?1 
forward
??d~1 
2 
3 
4 
stop
??1 
turn left
??d?1 
forward
??d~1 
2 
3 
4 
stop
??1 
turn left
??d?1 
forward
??d~1 
2 
3 
4 
stop
??1 
turn left
??d?1 
forward
??d~1 
2 
3 
4 
stop
??1 
turn left
??d?1 
forward
??d~1 
2 
3 
4 
stop
??1 
turn left
??d?1 
forward
??d~1 
2 
3 
4 
stop
??1 
turn left
??d?1 
forward
??d~1 
2 
3 
4 
stop
??1 
turn left
??d?1 
forward
??d~1 
2 
3 
4 
stop
??1 

```
