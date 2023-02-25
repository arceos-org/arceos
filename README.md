# ArceOS

An experimental modular operating system (or unikernel) written in Rust.

🚧 Working In Progress.

## Features & TODOs

* [x] Architecture: riscv64, aarch64
* [x] Platform: QEMU virt riscv64/aarch64
* [x] Multi-thread
* [x] Cooperative FIFO scheduler
* [x] VirtIO net/blk drivers
* [x] TCP net stack using [smoltcp](https://github.com/smoltcp-rs/smoltcp)
* [ ] File system
* [ ] Compatible with Linux apps
* [ ] Synchronization/Mutex
* [ ] Interrupt driven device I/O
* [ ] Async I/O
* [ ] Kernel preemption
* [ ] SMP

## Build & Run

### Rust apps

```bash
make ARCH=<arch> APP=<app> LOG=<log> NET=[on|off] FS=[on|off] run
```

Where `<arch>` can be one of `riscv64`, `aarch64`.

`<log>` can be one of `off`, `error`, `warn`, `info`, `debug`, `trace`.

`<app>` can be one of `helloworld`, `memtest`, `exception`, `multitask`, `httpclient`, `echoserver`. (See the [apps/](apps/) directory)

### C apps

```bash
make ARCH=<arch> APP=<app> LOG=<log> NET=[on|off] FS=[on|off] APP_LANG=c run
```

## Design

![](doc/ArceOS.svg)
