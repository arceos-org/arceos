use core::fmt;

use axfs_ng_vfs::{FileNode, Location, Metadata, NodePermission, VfsError, VfsResult, path::Path};
use axio::SeekFrom;
use lock_api::RawMutex;

use super::FsContext;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct FileFlags: u8 {
        const READ = 1;
        const WRITE = 2;
        const EXECUTE = 4;
        const APPEND = 8;
    }
}

/// Results returned by [`OpenOptions::open`].
pub enum OpenResult<M> {
    File(File<M>),
    Dir(Location<M>),
}

impl<M> OpenResult<M> {
    pub fn into_file(self) -> VfsResult<File<M>> {
        match self {
            Self::File(file) => Ok(file),
            Self::Dir(_) => Err(VfsError::EISDIR),
        }
    }

    pub fn into_dir(self) -> VfsResult<Location<M>> {
        match self {
            Self::Dir(dir) => Ok(dir),
            Self::File(_) => Err(VfsError::ENOTDIR),
        }
    }

    pub fn into_location(self) -> Location<M> {
        match self {
            Self::File(file) => file.inner,
            Self::Dir(dir) => dir,
        }
    }
}

/// Options and flags which can be used to configure how a file is opened.
#[derive(Clone)]
pub struct OpenOptions {
    // generic
    read: bool,
    write: bool,
    execute: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
    directory: bool,
    no_follow: bool,
    user: Option<(u32, u32)>,
    // system-specific
    custom_flags: i32,
    mode: u32,
}

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    pub fn new() -> Self {
        Self {
            // generic
            read: false,
            write: false,
            execute: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            directory: false,
            no_follow: false,
            user: None,
            // system-specific
            custom_flags: 0,
            mode: 0o666,
        }
    }

    /// Sets the option for read access.
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    /// Sets the option for write access.
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    /// Sets the option for execute access.
    pub fn execute(&mut self, execute: bool) -> &mut Self {
        self.execute = execute;
        self
    }

    /// Sets the option for the append mode.
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    /// Sets the option for truncating a previous file.
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    /// Sets the option to create a new file, or open it if it already exists.
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    /// Sets the option to create a new file, failing if it already exists.
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    /// Sets the option to open directory instead.
    pub fn directory(&mut self, directory: bool) -> &mut Self {
        self.directory = directory;
        self
    }

    /// Sets the option to not follow symlinks.
    pub fn no_follow(&mut self, no_follow: bool) -> &mut Self {
        self.no_follow = no_follow;
        self
    }

    /// Sets the user and group id to open the file with.
    pub fn user(&mut self, uid: u32, gid: u32) -> &mut Self {
        self.user = Some((uid, gid));
        self
    }

    /// Pass custom flags to the flags argument of open.
    pub fn custom_flags(&mut self, flags: i32) -> &mut Self {
        self.custom_flags = flags;
        self
    }

    /// Sets the mode bits that a new file will be created with.
    pub fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode = mode;
        self
    }

    pub fn open<M: RawMutex>(
        &self,
        context: &FsContext<M>,
        path: impl AsRef<Path>,
    ) -> VfsResult<OpenResult<M>> {
        self._open(context, path.as_ref())
    }

    fn _open<M: RawMutex>(&self, context: &FsContext<M>, path: &Path) -> VfsResult<OpenResult<M>> {
        if !self.is_valid() {
            return Err(VfsError::EINVAL);
        }
        let flags = self.to_flags()?;

        let loc = match context.resolve_parent(path.as_ref()) {
            Ok((parent, name)) => {
                let mut loc = parent.open_file(
                    &name,
                    &axfs_ng_vfs::OpenOptions {
                        create: self.create,
                        create_new: self.create_new,
                        permission: NodePermission::from_bits_truncate(self.mode as _),
                        user: self.user,
                    },
                )?;
                if !self.no_follow {
                    loc = context
                        .with_current_dir(parent)?
                        .try_resolve_symlink(loc, &mut 0)?;
                }
                loc
            }
            Err(VfsError::EINVAL) => {
                // root directory
                context.root_dir().clone()
            }
            Err(err) => return Err(err),
        };
        if self.directory {
            if flags.contains(FileFlags::WRITE) {
                return Err(VfsError::EISDIR);
            }
            loc.check_is_dir()?;
        }
        if self.truncate {
            loc.entry().as_file()?.set_len(0)?;
        }

        Ok(if loc.is_dir() {
            OpenResult::Dir(loc)
        } else {
            OpenResult::File(File::new(loc, flags))
        })
    }

    pub(crate) fn to_flags(&self) -> VfsResult<FileFlags> {
        Ok(match (self.read, self.write, self.append) {
            (true, false, false) => FileFlags::READ,
            (false, true, false) => FileFlags::WRITE,
            (true, true, false) => FileFlags::READ | FileFlags::WRITE,
            (false, _, true) => FileFlags::WRITE | FileFlags::APPEND,
            (true, _, true) => FileFlags::READ | FileFlags::WRITE | FileFlags::APPEND,
            (false, false, false) => return Err(VfsError::EINVAL),
        })
    }

    pub(crate) fn is_valid(&self) -> bool {
        if !self.read && !self.write && !self.append {
            return true;
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

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for OpenOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let OpenOptions {
            read,
            write,
            execute,
            append,
            truncate,
            create,
            create_new,
            directory,
            no_follow,
            user,
            custom_flags,
            mode,
        } = self;
        f.debug_struct("OpenOptions")
            .field("read", read)
            .field("write", write)
            .field("execute", execute)
            .field("append", append)
            .field("truncate", truncate)
            .field("create", create)
            .field("create_new", create_new)
            .field("directory", directory)
            .field("no_follow", no_follow)
            .field("user", user)
            .field("custom_flags", custom_flags)
            .field("mode", mode)
            .finish()
    }
}

/// Provides `std::fs::File`-like interface.
pub struct File<M> {
    inner: Location<M>,
    pub(crate) flags: FileFlags,

    position: u64,
}

impl<M: RawMutex> File<M> {
    pub(crate) fn new(inner: Location<M>, flags: FileFlags) -> Self {
        Self {
            inner,
            flags,
            position: 0,
        }
    }

    pub fn open(context: &FsContext<M>, path: impl AsRef<Path>) -> VfsResult<Self> {
        OpenOptions::new()
            .read(true)
            .open(context, path.as_ref())
            .and_then(OpenResult::into_file)
    }

    pub fn create(context: &FsContext<M>, path: impl AsRef<Path>) -> VfsResult<Self> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(context, path.as_ref())
            .and_then(OpenResult::into_file)
    }

    pub fn access(&self, cap: FileFlags) -> VfsResult<&FileNode<M>> {
        if self.flags.contains(cap) {
            self.inner.entry().as_file()
        } else {
            Err(VfsError::EBADF)
        }
    }

    pub fn inner(&self) -> &Location<M> {
        &self.inner
    }

    /// Attempts to sync OS-internal file content and metadata to disk.
    ///
    /// If `data_only` is `true`, only the file data is synced, not the
    /// metadata.
    pub fn sync(&self, data_only: bool) -> VfsResult<()> {
        self.inner.sync(data_only)
    }

    /// Truncates or extends the underlying file, updating the size of this file
    /// to become `size`.
    pub fn set_len(&self, size: u64) -> VfsResult<()> {
        self.access(FileFlags::WRITE)?.set_len(size)
    }

    /// Queries metadata about the underlying file.
    pub fn metadata(&self) -> VfsResult<Metadata> {
        self.access(FileFlags::READ)?;
        self.inner.metadata()
    }

    /// Reads a number of bytes starting from a given offset.
    pub fn read_at(&mut self, buf: &mut [u8], offset: u64) -> VfsResult<usize> {
        self.access(FileFlags::READ)?.read_at(buf, offset)
    }

    /// Writes a number of bytes starting from a given offset.
    pub fn write_at(&mut self, buf: &[u8], offset: u64) -> VfsResult<usize> {
        self.access(FileFlags::WRITE)?.write_at(buf, offset)
    }
}

impl<M: RawMutex> File<M> {
    /// Writes a number of bytes starting from the current position.
    pub fn write(&mut self, buf: &[u8]) -> VfsResult<usize> {
        if self.flags.contains(FileFlags::APPEND) {
            let (written, offset) = self.access(FileFlags::WRITE)?.append(buf)?;
            self.position = offset;
            Ok(written)
        } else {
            let n = self.write_at(buf, self.position)?;
            self.position += n as u64;
            Ok(n)
        }
    }
}

impl<M: RawMutex> axio::Read for File<M> {
    fn read(&mut self, buf: &mut [u8]) -> axio::Result<usize> {
        self.read_at(buf, self.position).inspect(|n| {
            self.position += *n as u64;
        })
    }
}

impl<M: RawMutex> axio::Write for File<M> {
    fn write(&mut self, buf: &[u8]) -> axio::Result<usize> {
        if self.flags.contains(FileFlags::APPEND) {
            self.access(FileFlags::WRITE)?
                .append(buf)
                .map(|(written, offset)| {
                    self.position = offset;
                    written
                })
        } else {
            self.write_at(buf, self.position).inspect(|n| {
                self.position += *n as u64;
            })
        }
    }

    fn flush(&mut self) -> axio::Result {
        Ok(())
    }
}

impl<M: RawMutex> axio::Seek for File<M> {
    fn seek(&mut self, pos: SeekFrom) -> axio::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(pos) => pos,
            SeekFrom::End(off) => {
                let size = self.access(FileFlags::empty())?.len()?;
                size.checked_add_signed(off)
                    .ok_or(VfsError::EINVAL)?
                    .clamp(0, size)
            }
            SeekFrom::Current(off) => {
                let size = self.access(FileFlags::empty())?.len()?;
                self.position
                    .checked_add_signed(off)
                    .ok_or(VfsError::EINVAL)?
                    .clamp(0, size)
            }
        };
        self.position = new_pos;
        Ok(new_pos)
    }
}
