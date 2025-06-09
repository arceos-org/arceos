//! # The ArceOS Async Standard Library
//!
//! [ArceOS]: https://github.com/arceos-org/arceos

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]

pub mod task;

mod io {
    pub type Result<T> = axio::Result<T>;
}
