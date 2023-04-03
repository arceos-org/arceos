mod stdio;

pub use axio::prelude;
pub use axio::{BufRead, BufReader, Error, Read, Result, Write};

pub use self::stdio::{stdin, stdout, Stdin, Stdout, __print_impl};
