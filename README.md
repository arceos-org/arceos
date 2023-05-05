# ArceOS

[![CI](https://github.com/rcore-os/arceos/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/rcore-os/arceos/actions)
[![Docs](https://img.shields.io/badge/docs-pages-green)](https://rcore-os.github.io/arceos/)

An experimental modular operating system (or unikernel) written in Rust.

ArceOS was inspired a lot by [Unikraft](https://github.com/unikraft/unikraft).

🚧 Working In Progress.

## Features & TODOs

* [x] Architecture: riscv64, aarch64
* [x] Platform: QEMU virt riscv64/aarch64
* [x] Multi-thread
* [x] Cooperative/preemptive scheduler
* [x] VirtIO net/blk/gpu drivers
* [x] TCP net stack using [smoltcp](https://github.com/smoltcp-rs/smoltcp)
* [x] Synchronization/Mutex
* [x] SMP scheduling with single run queue
* [x] File system
* [ ] Compatible with Linux apps
* [ ] Interrupt driven device I/O
* [ ] Async I/O

## Example apps

Example applications can be found in the [apps/](apps/) directory. All applications must at least depend on the following modules, while other modules are optional:

* [axruntime](modules/axruntime/): Bootstraping from the bare-metal environment, and initialization.
* [axhal](modules/axhal/): Hardware abstraction layer, provides unified APIs for cross-platform.
* [axconfig](modules/axconfig/): Platform constants and kernel parameters, such as physical memory base, kernel load addresses, stack size, etc.
* [axlog](modules/axlog/): Multi-level log definition and printing.

The currently supported applications (Rust), as well as their dependent modules and features, are shown in the following table:

| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [helloworld](apps/helloworld/) | | | A minimal app that just prints a string |
| [exception](apps/exception/) | | paging | Exception handling test |
| [memtest](apps/memtest/) | axalloc | alloc, paging | Dynamic memory allocation test |
| [display](apps/display/) | axalloc, axdisplay | alloc, paging, display | Graphic/GUI test |
| [yield](apps/task/yield/) | axalloc, axtask | alloc, paging, multitask, sched_fifo | Multi-threaded yielding test |
| [parallel](apps/task/parallel/) | axalloc, axtask | alloc, paging, multitask, sched_fifo | Parallel computing test (to test synchronization & mutex) |
| [sleep](apps/task/sleep/) | axalloc, axtask | alloc, paging, multitask, sched_fifo | Thread sleeping test |
| [shell](apps/fs/shell/) | axalloc, axdriver, axfs | alloc, paging, fs | A simple shell that responds to filesystem operations |
| [httpclient](apps/net/httpclient/) | axalloc, axdriver, axnet | alloc, paging, net | A simple client that sends an HTTP request and then prints the response |
| [echoserver](apps/net/echoserver/) | axalloc, axdriver, axnet, axtask | alloc, paging, net, multitask | A multi-threaded TCP server that reverses messages sent by the client  |
| [httpserver](apps/net/httpserver/) | axalloc, axdriver, axnet, axtask | alloc, paging, net, multitask | A multi-threaded HTTP server that serves a static web page |

## Build & Run

### Install build dependencies

Install [cargo-binutils](https://github.com/rust-embedded/cargo-binutils) to use `rust-objcopy` and `rust-objdump` tools:

```bash
cargo install cargo-binutils
```

### Example apps

```bash
# in arceos directory
make A=path/to/app ARCH=<arch> LOG=<log> NET=[y|n] FS=[y|n]
```

Where `<arch>` should be one of `riscv64`, `aarch64`.

`<log>` should be one of `off`, `error`, `warn`, `info`, `debug`, `trace`.

`path/to/app` is the relative path to the example application.

More arguments and targets can be found in [Makefile](Makefile).

For example, to run the [httpserver](apps/net/httpserver/) on `qemu-system-aarch64` with 4 cores:

```bash
make A=apps/net/httpserver ARCH=aarch64 LOG=info NET=y SMP=4 run
```

### Your custom apps

#### Rust

1. Create a new rust package with `no_std` and `no_main` environment.
2. Add `libax` dependency and features to enable to `Cargo.toml`:

    ```toml
    [dependencies]
    libax = { path = "/path/to/arceos/ulib/libax", features = ["..."] }
    ```

3. Call library functions from `libax` in your code, like the [helloworld](apps/helloworld/) example.
4. Build your application with ArceOS, by running the `make` command in the application directory:

    ```bash
    # in app directory
    make -C /path/to/arceos A=$(pwd) ARCH=<arch> run
    # more args: LOG=<log> SMP=<smp> NET=[y|n] ...
    ```

    All arguments and targets are the same as above.

#### C

1. Create `axbuild.mk` and `features.txt` in your project:

    ```bash
    app/
    ├── foo.c
    ├── bar.c
    ├── axbuild.mk      # optional, if there is only one `main.c`
    └── features.txt    # optional, if only use default features
    ```

2. Add build targets to `axbuild.mk`, add features to enable to `features.txt` (see this [example](apps/c/sqlite3/)):

    ```bash
    # in axbuild.mk
    app-objs := foo.o bar.o
    ```

    ```bash
    # in features.txt
    default
    alloc
    paging
    net
    ```

3. Build your application with ArceOS, by running the `make` command in the application directory:

    ```bash
    # in app directory
    make -C /path/to/arceos A=$(pwd) ARCH=<arch> run
    # more args: LOG=<log> SMP=<smp> NET=[y|n] ...
    ```

## Design

![](doc/ArceOS.svg)



## 进程支持下的应用程序启动

1. 应用程序文件准备：

   若仅有应用程序源码，则需要将准备运行的应用程序与用户库进行联合编译，生成可执行文件。编译方式可以参考`rCore`（[rcore-os/rCore-Tutorial-v3: Let's write an OS which can run on RISC-V in Rust from scratch! (github.com)](https://github.com/rcore-os/rCore-Tutorial-v3)）的`user`库编译方式。

   比赛中测例通过联合编译之后也会生成可执行文件。

   生成流程如下：

   1. 在`rCore`的`user`库下`bin`文件夹新建一个名为`helloworld.rs`的文件
   2. 在该文件中编写您想运行的应用程序源码。
   3. 在`user`路径下执行`make build`指令。
   4. `user/target/riscv64-unknown-none-elf/release/helloworld`即为所生成的可执行文件。

   由于当前未引入文件系统支持，因此采用固定路径链接可执行文件。请将预备执行的可执行文件拷贝在`arceos`的`apps/helloworld`路径下，由于后续不打算在固定路径链接上做拓展，因此写死初始化运行程序的文件名必须为`helloworld`。

   若需要引入多个文件，在将多个文件放入到对应目录下后，还需要修改`axruntime/src/link_app.S`与源码中某些部分。由于不打算后续扩展，因此当前写死仅支持两个文件同时链接加载。初始化应用为`helloworld`，另一个应用为`second`。

2. 启动应用程序运行指令：

   在根目录下运行

   ```rust
   make A=apps/helloworld ARCH=riscv64 LOG=info SMP=1 run
   ```

   即可启动任务调度器，反复检查当前是否有可执行的任务。在执行完所有任务之后，任务调度器不会退出，而是继续循环，类似于`shell`的执行逻辑。

3. 若想运行其他内容的应用程序，请在原先`helloworld`的源码上进行修改，引入其他系统调用之后再次编译生成可执行文件并且拷贝到对应目录。
4. 当前由于未支持文件系统，上述操作略显冗余。之后会引入文件系统支持，使得流程更为简便。
