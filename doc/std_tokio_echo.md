# INTRODUCTION
| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [tokio-echo](../apps/std/tokio-echo/) | axalloc, axdriver, axnet | net, multitask | A simple http client based on [`tokio`](https://tokio.rs/) that sends an HTTP request and then prints the response |

This example is to show ArceOS's support for Rust `std`. 
It relies on our modified [Rust std standard library](https://github.com/arceos-os/rust.git) and needs to initialize submodule before compiling:
```bash
git submodule update --init --recursive
```


# RUN
```bash
make A=apps/std/tokio-echo SMP=1 NET=y LOG=debug STD=y run
```

