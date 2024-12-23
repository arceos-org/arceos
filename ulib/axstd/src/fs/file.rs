use crate::io::{Result, SeekFrom, prelude::*};
use core::fmt;

use arceos_api::fs as api;

/// A structure representing a type of file with accessors for each file type.
/// It is returned by [`Metadata::file_type`] method.
pub type FileType = api::AxFileType;

/// Representation of the various permissions on a file.
pub type Permissions = api::AxFilePerm;

/// An object providing access to an open file on the filesystem.
pub struct File {
    inner: api::AxFileHandle,
}

/// Metadata information about a file.
pub struct Metadata(api::AxFileAttr);

/// Options and flags which can be used to configure how a file is opened.
#[derive(Clone, Debug)]
pub struct OpenOptions(api::AxOpenOptions);

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    pub const fn new() -> Self {
        OpenOptions(api::AxOpenOptions::new())
    }

    /// Sets the option for read access.
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.0.read(read);
        self
    }

    /// Sets the option for write access.
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.0.write(write);
        self
    }

    /// Sets the option for the append mode.
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.0.append(append);
        self
    }

    /// Sets the option for truncating a previous file.
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.0.truncate(truncate);
        self
    }

    /// Sets the option to create a new file, or open it if it already exists.
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.0.create(create);
        self
    }

    /// Sets the option to create a new file, failing if it already exists.
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.0.create_new(create_new);
        self
    }

    /// Opens a file at `path` with the options specified by `self`.
    pub fn open(&self, path: &str) -> Result<File> {
        api::ax_open_file(path, &self.0).map(|inner| File { inner })
    }
}

impl Metadata {
    /// Returns the file type for this metadata.
    pub const fn file_type(&self) -> FileType {
        self.0.file_type()
    }

    /// Returns `true` if this metadata is for a directory. The
    /// result is mutually exclusive to the result of
    /// [`Metadata::is_file`].
    pub const fn is_dir(&self) -> bool {
        self.0.is_dir()
    }

    /// Returns `true` if this metadata is for a regular file. The
    /// result is mutually exclusive to the result of
    /// [`Metadata::is_dir`].
    pub const fn is_file(&self) -> bool {
        self.0.is_file()
    }

    /// Returns the size of the file, in bytes, this metadata is for.
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> u64 {
        self.0.size()
    }

    /// Returns the permissions of the file this metadata is for.
    pub const fn permissions(&self) -> Permissions {
        self.0.perm()
    }

    /// Returns the total size of this file in bytes.
    pub const fn size(&self) -> u64 {
        self.0.size()
    }

    /// Returns the number of blocks allocated to the file, in 512-byte units.
    pub const fn blocks(&self) -> u64 {
        self.0.blocks()
    }
}

impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Metadata")
            .field("file_type", &self.file_type())
            .field("is_dir", &self.is_dir())
            .field("is_file", &self.is_file())
            .field("permissions", &self.permissions())
            .finish_non_exhaustive()
    }
}

impl File {
    /// Attempts to open a file in read-only mode.
    pub fn open(path: &str) -> Result<Self> {
        OpenOptions::new().read(true).open(path)
    }

    /// Opens a file in write-only mode.
    pub fn create(path: &str) -> Result<Self> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
    }

    /// Creates a new file in read-write mode; error if the file exists.
    pub fn create_new(path: &str) -> Result<Self> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
    }

    /// Returns a new OpenOptions object.
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    /// Truncates or extends the underlying file, updating the size of
    /// this file to become `size`.
    pub fn set_len(&self, size: u64) -> Result<()> {
        api::ax_truncate_file(&self.inner, size)
    }

    /// Queries metadata about the underlying file.
    pub fn metadata(&self) -> Result<Metadata> {
        api::ax_file_attr(&self.inner).map(Metadata)
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        api::ax_read_file(&mut self.inner, buf)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        api::ax_write_file(&mut self.inner, buf)
    }

    fn flush(&mut self) -> Result<()> {
        api::ax_flush_file(&self.inner)
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        api::ax_seek_file(&mut self.inner, pos)
    }
}
