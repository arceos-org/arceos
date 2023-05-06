//! Low-level filesystem operations.
//! 这个接口定义提供了打开/关闭文件和目录,读写文件,创建/删除文件和目录等操作。使用File和Directory结构表示打开的文件和目录,并通过这两个结构提供具体方法。
//! OpenOptions用于指定打开文件和目录的选项,如ONLY,CREATE,APPEND等。通过实现From和perm_to_cap,可以从OpenOptions得到访问权限,用于构造WithCap。
//! WithCap在每次方法调用时带入访问权限,使得对底层节点的操作始终带有正确的权限检查。
//! 整体来说,这个接口定义提供了统一和安全地访问文件系统的方法。通过权限检查和偏移量维护,可以避免许多低级错误。

use axerrno::{ax_err, ax_err_type, AxResult};
use axfs_vfs::{VfsError, VfsNodeRef};
use axio::SeekFrom;
use capability::{Cap, WithCap};
use core::fmt;

pub type FileType = axfs_vfs::VfsNodeType;      // 文件类型
pub type DirEntry = axfs_vfs::VfsDirEntry;      // 目录项
pub type FileAttr = axfs_vfs::VfsNodeAttr;      // 文件属性
pub type FilePerm = axfs_vfs::VfsNodePerm;      // 文件权限

/// Filesystem operations. 打开的文件
pub struct File {
    node: WithCap<VfsNodeRef>,      // 包含访问权限的文件节点引用(Inner+Cap,Cap就是三种权限的bitflag)
    is_append: bool,                // 是否以追加模式打开
    offset: u64,
}

/// Directory operations. 打开的目录
pub struct Directory {
    node: WithCap<VfsNodeRef>,      // 包含访问权限的节点引用
    entry_idx: usize,               // 目录项索引
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
    /// 以相对/绝对路径打开文件
    pub fn open(path: &str, opts: &OpenOptions) -> AxResult<Self> {
        Self::_open_at(None, path, opts)
    }
    /// 截断文件到指定大小
    pub fn truncate(&self, size: u64) -> AxResult {
        self.node.access(Cap::WRITE)?.truncate(size)?;
        Ok(())
    }
    /// 读文件, 返回读取的字节数
    pub fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        let node = self.node.access(Cap::READ)?;
        let read_len = node.read_at(self.offset, buf)?;
        self.offset += read_len as u64;
        Ok(read_len)
    }
    /// 写文件, 返回写入的字节数
    pub fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        let node = self.node.access(Cap::WRITE)?;
        if self.is_append {
            self.offset = self.get_attr()?.size();          // 如果是追加模式, 则会将文件指针移动到文件末尾
        };
        let write_len = node.write_at(self.offset, buf)?;
        self.offset += write_len as u64;
        Ok(write_len)
    }
    /// 清空缓冲区, 将缓冲区中的数据写入磁盘
    pub fn flush(&self) -> AxResult {
        self.node.access(Cap::WRITE)?.fsync()?;
        Ok(())
    }
    /// 设置文件指针位置
    pub fn seek(&mut self, pos: SeekFrom) -> AxResult<u64> {
        let size = self.get_attr()?.size();
        let new_offset = match pos {
            SeekFrom::Start(pos) => Some(pos),
            SeekFrom::Current(off) => self.offset.checked_add_signed(off),
            SeekFrom::End(off) => size.checked_add_signed(off),
        }
            .ok_or_else(|| ax_err_type!(InvalidInput))?;        // 如果是Some(x), 则返回Ok(x), 否则返回InvalidInput错误
        self.offset = new_offset;
        Ok(new_offset)
    }
    /// 获取文件属性
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
    /// 获取目录项
    fn access_at(&self, path: &str) -> AxResult<Option<&VfsNodeRef>> {
        if path.starts_with('/') {
            Ok(None)
        } else {
            Ok(Some(self.node.access(Cap::EXECUTE)?))
        }
    }
    /// 以相对/绝对路径打开目录
    pub fn open_dir(path: &str, opts: &OpenOptions) -> AxResult<Self> {
        Self::_open_dir_at(None, path, opts)
    }
    /// 打开目录项
    pub fn open_dir_at(&self, path: &str, opts: &OpenOptions) -> AxResult<Self> {
        Self::_open_dir_at(self.access_at(path)?, path, opts)
    }
    /// 打开目录项的文件
    pub fn open_file_at(&self, path: &str, opts: &OpenOptions) -> AxResult<File> {
        File::_open_at(self.access_at(path)?, path, opts)
    }
    /// 创建文件
    pub fn create_file(&self, path: &str) -> AxResult<VfsNodeRef> {
        crate::root::create_file(self.access_at(path)?, path)
    }
    /// 创建子目录
    pub fn create_dir(&self, path: &str) -> AxResult {
        crate::root::create_dir(self.access_at(path)?, path)
    }
    /// 删除文件
    pub fn remove_file(&self, path: &str) -> AxResult {
        crate::root::remove_file(self.access_at(path)?, path)
    }
    /// 删除子目录
    pub fn remove_dir(&self, path: &str) -> AxResult {
        crate::root::remove_dir(self.access_at(path)?, path)
    }
    //获取目录
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
