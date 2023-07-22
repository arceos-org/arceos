//! Traits, helpers, and type definitions for core I/O functionality.

mod stdio;

pub use axio::prelude;
pub use axio::{BufRead, BufReader, Error, PollState, Read, Result, Seek, SeekFrom, Write};

pub use self::stdio::{stdin, stdout, Stdin, Stdout, __print_impl};
