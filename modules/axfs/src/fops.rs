//! Low-level filesystem operations.

use axerrno::{ax_err, AxResult};
use axfs_vfs::{VfsError, VfsNodeRef};
use core::fmt;

pub type FileType = axfs_vfs::VfsNodeType;
pub type DirEntry = axfs_vfs::VfsDirEntry;
pub type FileAttr = axfs_vfs::VfsNodeAttr;
pub type FilePerm = axfs_vfs::VfsNodePerm;

pub struct File {
    node: VfsNodeRef,
    is_append: bool,
    offset: u64,
}

pub struct Directory {
    node: VfsNodeRef,
    entry_idx: usize,
}

#[derive(Clone)]
pub struct OpenOptions {
    // generic
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
    // system-specific
    _custom_flags: i32,
    _mode: u32,
}

impl OpenOptions {
    pub const fn new() -> Self {
        Self {
            // generic
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            // system-specific
            _custom_flags: 0,
            _mode: 0o666,
        }
    }
    pub fn read(&mut self, read: bool) {
        self.read = read;
    }
    pub fn write(&mut self, write: bool) {
        self.write = write;
    }
    pub fn append(&mut self, append: bool) {
        self.append = append;
    }
    pub fn truncate(&mut self, truncate: bool) {
        self.truncate = truncate;
    }
    pub fn create(&mut self, create: bool) {
        self.create = create;
    }
    pub fn create_new(&mut self, create_new: bool) {
        self.create_new = create_new;
    }

    const fn is_valid(&self) -> bool {
        if !self.read && !self.write && !self.append {
            return false;
        }
        match (self.write, self.append) {
            (true, false) => {}
            (false, false) => {
                if self.truncate || self.create || self.create_new {
                    return false;
                }
            }
            (_, true) => {
                if self.truncate && !self.create_new {
                    return false;
                }
            }
        }
        true
    }
}

impl File {
    pub fn open(path: &str, opts: &OpenOptions) -> AxResult<Self> {
        debug!("open file: {} {:?}", path, opts);
        if !opts.is_valid() {
            return ax_err!(InvalidInput);
        }

        let node_option = crate::root::lookup(path);
        let node = if opts.create || opts.create_new {
            match node_option {
                Ok(node) => {
                    // already exists
                    if opts.create_new {
                        return ax_err!(AlreadyExists);
                    }
                    node
                }
                // not exists, create new
                Err(VfsError::NotFound) => crate::root::create_file(path)?,
                Err(e) => return Err(e),
            }
        } else {
            // just open the existing
            node_option?
        };

        let attr = node.get_attr()?;
        if attr.is_dir()
            && (opts.create || opts.create_new || opts.write || opts.append || opts.truncate)
        {
            return ax_err!(IsADirectory);
        }
        node.open()?;
        if opts.truncate {
            node.truncate(0)?;
        }

        Ok(Self {
            node,
            is_append: opts.append,
            offset: 0,
        })
    }

    pub fn truncate(&self, size: u64) -> AxResult {
        self.node.truncate(size)?;
        Ok(())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        let read_len = self.node.read_at(self.offset, buf)?;
        self.offset += read_len as u64;
        Ok(read_len)
    }

    pub fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        if self.is_append {
            self.offset = self.node.get_attr()?.size();
        };
        let write_len = self.node.write_at(self.offset, buf)?;
        self.offset += write_len as u64;
        Ok(write_len)
    }

    pub fn flush(&self) -> AxResult {
        self.node.fsync()?;
        Ok(())
    }

    pub fn get_attr(&self) -> AxResult<FileAttr> {
        self.node.get_attr()
    }
}

impl Directory {
    pub fn open_dir(path: &str, opts: &OpenOptions) -> AxResult<Self> {
        debug!("open dir: {}", path);
        if !opts.read {
            return ax_err!(InvalidInput);
        }
        if opts.create || opts.create_new || opts.write || opts.append || opts.truncate {
            return ax_err!(InvalidInput);
        }

        let node = crate::root::lookup(path)?;
        if node.get_attr()?.is_dir() {
            node.open()?;
            Ok(Self { node, entry_idx: 0 })
        } else {
            ax_err!(NotADirectory)
        }
    }

    pub fn read_dir(&mut self, dirents: &mut [DirEntry]) -> AxResult<usize> {
        let n = self.node.read_dir(self.entry_idx, dirents)?;
        self.entry_idx += n;
        Ok(n)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        self.node.release().ok();
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        self.node.release().ok();
    }
}

impl fmt::Debug for OpenOptions {
    #[allow(unused_assignments)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut written = false;
        macro_rules! fmt_opt {
            ($field: ident, $label: literal) => {
                if self.$field {
                    if written {
                        write!(f, " | ")?;
                    }
                    write!(f, $label)?;
                    written = true;
                }
            };
        }
        fmt_opt!(read, "READ");
        fmt_opt!(write, "WRITE");
        fmt_opt!(append, "APPEND");
        fmt_opt!(truncate, "TRUNC");
        fmt_opt!(create, "CREATE");
        fmt_opt!(create_new, "CREATE_NEW");
        Ok(())
    }
}
