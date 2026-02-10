# axtask

[![Crates.io](https://img.shields.io/crates/v/axtask)](https://crates.io/crates/axtask)
[![Docs.rs](https://docs.rs/axtask/badge.svg)](https://docs.rs/axtask)

[ArceOS](https://github.com/arceos-org/arceos) task management module.

This module provides primitives for task management, including task creation,
scheduling, sleeping, termination, etc. The scheduler algorithm is configurable
by cargo features.

## Features

- `multitask`: Enable multi-task support with complex scheduling and more task-related APIs.
- `irq`: Enable timer-based APIs such as `sleep`, `sleep_until`, and `WaitQueue::wait_timeout`.
- `preempt`: Enable preemptive scheduling.
- `sched-fifo`: Use the FIFO cooperative scheduler (enables `multitask`).
- `sched-rr`: Use the Round-robin preemptive scheduler (enables `multitask` and `preempt`).
- `sched-cfs`: Use the Completely Fair Scheduler (enables `multitask` and `preempt`).
- `tls`: Enable kernel space thread-local storage support.
- `smp`: Enable SMP (symmetric multiprocessing) support.

## License

This project is licensed under GPL-3.0-or-later OR Apache-2.0 OR MulanPSL-2.0.
