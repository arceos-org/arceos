//! Filesystem manipulation operations.

pub use axfs::api::{canonicalize, metadata, read, read_to_string, remove_file, write};
pub use axfs::api::{create_dir, create_dir_all, link_file, read_dir, remove_dir, symbolic_link};
pub use axfs::api::{DirEntry, File, FileType, Metadata, OpenOptions, Permissions, ReadDir};
