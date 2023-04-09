//! Low-level filesystem operations.

use axerrno::{ax_err, ax_err_type, AxResult};
use axfs_vfs::{VfsError, VfsNodeRef};
use axio::SeekFrom;
use capability::{Cap, WithCap};
use core::fmt;

pub type FileType = axfs_vfs::VfsNodeType;
pub type DirEntry = axfs_vfs::VfsDirEntry;
pub type FileAttr = axfs_vfs::VfsNodeAttr;
pub type FilePerm = axfs_vfs::VfsNodePerm;

pub struct File {
    node: WithCap<VfsNodeRef>,
    is_append: bool,
    offset: u64,
}

pub struct Directory {
    node: WithCap<VfsNodeRef>,
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
    fn _open_at(dir: Option<&VfsNodeRef>, path: &str, opts: &OpenOptions) -> AxResult<Self> {
        debug!("open file: {} {:?}", path, opts);
        if !opts.is_valid() {
            return ax_err!(InvalidInput);
        }

        let node_option = crate::root::lookup(dir, path);
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
                Err(VfsError::NotFound) => crate::root::create_file(dir, path)?,
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
        let access_cap = opts.into();
        if !perm_to_cap(attr.perm()).contains(access_cap) {
            return ax_err!(PermissionDenied);
        }

        node.open()?;
        if opts.truncate {
            node.truncate(0)?;
        }
        Ok(Self {
            node: WithCap::new(node, access_cap),
            is_append: opts.append,
            offset: 0,
        })
    }

    pub fn open(path: &str, opts: &OpenOptions) -> AxResult<Self> {
        Self::_open_at(None, path, opts)
    }

    pub fn truncate(&self, size: u64) -> AxResult {
        self.node.access(Cap::WRITE)?.truncate(size)?;
        Ok(())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        let node = self.node.access(Cap::READ)?;
        let read_len = node.read_at(self.offset, buf)?;
        self.offset += read_len as u64;
        Ok(read_len)
    }

    pub fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        let node = self.node.access(Cap::WRITE)?;
        if self.is_append {
            self.offset = self.get_attr()?.size();
        };
        let write_len = node.write_at(self.offset, buf)?;
        self.offset += write_len as u64;
        Ok(write_len)
    }

    pub fn flush(&self) -> AxResult {
        self.node.access(Cap::WRITE)?.fsync()?;
        Ok(())
    }

    pub fn seek(&mut self, pos: SeekFrom) -> AxResult<u64> {
        let size = self.get_attr()?.size();
        let new_offset = match pos {
            SeekFrom::Start(pos) => Some(pos),
            SeekFrom::Current(off) => self.offset.checked_add_signed(off),
            SeekFrom::End(off) => size.checked_add_signed(off),
        }
        .ok_or(ax_err_type!(InvalidInput))?;
        if new_offset > size {
            return ax_err!(InvalidInput);
        }
        self.offset = new_offset;
        Ok(new_offset)
    }

    pub fn get_attr(&self) -> AxResult<FileAttr> {
        self.node.access(Cap::empty())?.get_attr()
    }
}

impl Directory {
    fn _open_dir_at(dir: Option<&VfsNodeRef>, path: &str, opts: &OpenOptions) -> AxResult<Self> {
        debug!("open dir: {}", path);
        if !opts.read {
            return ax_err!(InvalidInput);
        }
        if opts.create || opts.create_new || opts.write || opts.append || opts.truncate {
            return ax_err!(InvalidInput);
        }

        let node = crate::root::lookup(dir, path)?;
        let attr = node.get_attr()?;
        if !attr.is_dir() {
            return ax_err!(NotADirectory);
        }
        let access_cap = opts.into();
        if !perm_to_cap(attr.perm()).contains(access_cap) {
            return ax_err!(PermissionDenied);
        }

        node.open()?;
        Ok(Self {
            node: WithCap::new(node, access_cap),
            entry_idx: 0,
        })
    }

    fn access_at(&self, path: &str) -> AxResult<Option<&VfsNodeRef>> {
        if path.starts_with('/') {
            Ok(None)
        } else {
            Ok(Some(self.node.access(Cap::EXECUTE)?))
        }
    }

    pub fn open_dir(path: &str, opts: &OpenOptions) -> AxResult<Self> {
        Self::_open_dir_at(None, path, opts)
    }

    pub fn open_dir_at(&self, path: &str, opts: &OpenOptions) -> AxResult<Self> {
        Self::_open_dir_at(self.access_at(path)?, path, opts)
    }

    pub fn open_file_at(&self, path: &str, opts: &OpenOptions) -> AxResult<File> {
        File::_open_at(self.access_at(path)?, path, opts)
    }

    pub fn create_file(&self, path: &str) -> AxResult<VfsNodeRef> {
        crate::root::create_file(self.access_at(path)?, path)
    }

    pub fn create_dir(&self, path: &str) -> AxResult {
        crate::root::create_dir(self.access_at(path)?, path)
    }

    pub fn remove_file(&self, path: &str) -> AxResult {
        crate::root::remove_file(self.access_at(path)?, path)
    }

    pub fn remove_dir(&self, path: &str) -> AxResult {
        crate::root::remove_dir(self.access_at(path)?, path)
    }

    pub fn read_dir(&mut self, dirents: &mut [DirEntry]) -> AxResult<usize> {
        let n = self
            .node
            .access(Cap::READ)?
            .read_dir(self.entry_idx, dirents)?;
        self.entry_idx += n;
        Ok(n)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe { self.node.access_unchecked().release().ok() };
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        unsafe { self.node.access_unchecked().release().ok() };
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

impl From<&OpenOptions> for Cap {
    fn from(opts: &OpenOptions) -> Cap {
        let mut cap = Cap::empty();
        if opts.read {
            cap |= Cap::READ;
        }
        if opts.write | opts.append {
            cap |= Cap::WRITE;
        }
        cap
    }
}

fn perm_to_cap(perm: FilePerm) -> Cap {
    let mut cap = Cap::empty();
    if perm.owner_readable() {
        cap |= Cap::READ;
    }
    if perm.owner_writable() {
        cap |= Cap::WRITE;
    }
    if perm.owner_executable() {
        cap |= Cap::EXECUTE;
    }
    cap
}
