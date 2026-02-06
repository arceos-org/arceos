# axsync

[![Crates.io](https://img.shields.io/crates/v/axsync)](https://crates.io/crates/axsync)
[![Docs.rs](https://docs.rs/axsync/badge.svg)](https://docs.rs/axsync)

[ArceOS](https://github.com/arceos-org/arceos) synchronization primitives.

## Primitives

- **Mutex**: A mutual exclusion primitive. With the `multitask` feature, it uses task-aware locking; otherwise it is an alias of `kspin::SpinNoIrq`.
- **spin**: Re-export of the [kspin](https://crates.io/crates/kspin) crate (spinlocks).

## Features

- `multitask`: Enable multi-threaded support. When enabled, `Mutex` uses blocking that cooperates with the task scheduler; when disabled, `Mutex` is a spinlock.

## License

This project is licensed under GPL-3.0-or-later OR Apache-2.0 OR MulanPSL-2.0.
