use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::{fmt, num::NonZeroUsize, ops::Range};

use axfs_ng_vfs::{FileNode, Location, NodePermission, VfsError, VfsResult, path::Path};
use axio::SeekFrom;
use axsync::Mutex;
use lock_api::RawMutex;
use log::warn;
use lru::LruCache;
use weak_map::WeakMap;

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
pub enum OpenResult<M: RawMutex> {
    File(File<M>),
    Dir(Location<M>),
}

impl<M: RawMutex> OpenResult<M> {
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
    direct: bool,
    user: Option<(u32, u32)>,
    // system-specific
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
            direct: false,
            user: None,
            // system-specific
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

    /// Sets the option to open the file with direct I/O.\
    pub fn direct(&mut self, direct: bool) -> &mut Self {
        self.direct = direct;
        self
    }

    /// Sets the user and group id to open the file with.
    pub fn user(&mut self, uid: u32, gid: u32) -> &mut Self {
        self.user = Some((uid, gid));
        self
    }

    /// Sets the mode bits that a new file will be created with.
    pub fn mode(&mut self, mode: u32) -> &mut Self {
        self.mode = mode;
        self
    }

    pub fn open(
        &self,
        context: &FsContext<axsync::RawMutex>,
        path: impl AsRef<Path>,
    ) -> VfsResult<OpenResult<axsync::RawMutex>> {
        self._open(context, path.as_ref())
    }

    fn _open(
        &self,
        context: &FsContext<axsync::RawMutex>,
        path: &Path,
    ) -> VfsResult<OpenResult<axsync::RawMutex>> {
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
            // TODO(mivik): is this correct?
            let backend = if loc.filesystem().is_cacheable() && !self.direct {
                FileBackend::new_cached(loc)
            } else {
                FileBackend::new_direct(loc)
            };
            OpenResult::File(File::new(backend, flags))
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
            direct,
            user,
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
            .field("direct", direct)
            .field("user", user)
            .field("mode", mode)
            .finish()
    }
}

const PAGE_SIZE: usize = 4096;

struct PageCache {
    data: Box<[u8; PAGE_SIZE]>,
    dirty: bool,
}
impl PageCache {
    fn new() -> Self {
        let mut data = Box::new_uninit();
        unsafe {
            core::ptr::write_bytes(data.as_mut_ptr(), 0, 1);
        }
        Self {
            data: unsafe { data.assume_init() },
            dirty: false,
        }
    }
}
impl Drop for PageCache {
    fn drop(&mut self) {
        if self.dirty {
            warn!("dirty page dropped without flushing");
        }
    }
}

/// Map of all cached files.
///
/// For direct file, we don't need to ensure
static CACHED_FILE_TABLE: Mutex<WeakMap<usize, Weak<CachedFile<axsync::RawMutex>>>> =
    Mutex::new(WeakMap::new());

pub struct CachedFile<M: RawMutex> {
    inner: Location<M>,
    page_cache: Mutex<LruCache<u32, PageCache>>,
}

impl<M: RawMutex> CachedFile<M> {
    pub fn new(inner: Location<M>) -> Self {
        Self {
            inner,
            // TODO(mivik): tune this value
            page_cache: Mutex::new(LruCache::new(NonZeroUsize::new(64).unwrap())),
        }
    }

    fn evict_cache(file: &FileNode<M>, pn: u32, mut page: PageCache) -> VfsResult<()> {
        if page.dirty {
            let page_start = pn as u64 * PAGE_SIZE as u64;
            let len = (file.len()? - page_start).min(PAGE_SIZE as u64) as usize;
            file.write_at(&page.data[..len], page_start)?;
            page.dirty = false;
        }
        Ok(())
    }

    fn with_pages_or_else<T>(
        &self,
        range: Range<u64>,
        mut initial: T,
        page_before: impl FnOnce(&FileNode<M>) -> VfsResult<()>,
        mut page_each: impl FnMut(T, &mut PageCache, Range<usize>) -> VfsResult<T>,
    ) -> VfsResult<T> {
        let file = self.inner.entry().as_file()?;
        page_before(file)?;
        let start_page = (range.start / PAGE_SIZE as u64) as u32;
        let end_page = range.end.div_ceil(PAGE_SIZE as u64) as u32;
        let mut page_offset = (range.start % PAGE_SIZE as u64) as usize;
        for pn in start_page..end_page {
            let page_start = pn as u64 * PAGE_SIZE as u64;

            let mut guard = self.page_cache.lock();
            let page = if let Some(page) = guard.get_mut(&pn) {
                page
            } else {
                if guard.len() == guard.cap().get() {
                    // Cache is full, remove the least recently used page
                    if let Some((pn, page)) = guard.pop_lru() {
                        Self::evict_cache(file, pn, page)?;
                    }
                }

                // Page not in cache, read it
                let mut page = PageCache::new();
                file.read_at(page.data.as_mut(), page_start)?;
                guard.put(pn, page);
                guard.get_mut(&pn).unwrap()
            };

            initial = page_each(
                initial,
                page,
                page_offset..(range.end - page_start).min(PAGE_SIZE as u64) as usize,
            )?;
            page_offset = 0;
        }

        Ok(initial)
    }

    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> VfsResult<usize> {
        let len = self.inner.len()?;
        let end = (offset + buf.len() as u64).min(len);
        if end <= offset {
            return Ok(0);
        }
        self.with_pages_or_else(
            offset..end,
            (buf, 0),
            |_| Ok(()),
            |(buf, read), page, range| {
                let len = range.end - range.start;
                buf[..len].copy_from_slice(&page.data[range.start..range.end]);
                Ok((&mut buf[len..], read + len))
            },
        )
        .map(|(_, read)| read)
    }

    pub fn write_at(&self, buf: &[u8], offset: u64) -> VfsResult<usize> {
        let end = offset + buf.len() as u64;
        self.with_pages_or_else(
            offset..end,
            (buf, 0),
            |file| {
                if end > file.len()? {
                    file.set_len(end)?;
                }
                Ok(())
            },
            |(buf, written), page, range| {
                let len = range.end - range.start;
                page.data[range.start..range.end].copy_from_slice(&buf[..len]);
                page.dirty = true;
                Ok((&buf[len..], written + len))
            },
        )
        .map(|(_, written)| written)
    }

    pub fn set_len(&self, len: u64) -> VfsResult<()> {
        let file = self.inner.entry().as_file()?;
        let old_len = file.len()?;
        file.set_len(len)?;

        let old_last_page = (old_len / PAGE_SIZE as u64) as u32;
        let new_last_page = (len / PAGE_SIZE as u64) as u32;
        if old_len < len {
            // The file was extended, we need to evict the last page
            let mut guard = self.page_cache.lock();
            if let Some(page) = guard.pop(&old_last_page) {
                Self::evict_cache(file, old_last_page, page)?;
            }
        } else if old_last_page > new_last_page {
            // For truncating, we need to remove all pages that are beyond the
            // new length
            // TODO(mivik): can this be more efficient?
            let mut guard = self.page_cache.lock();
            let keys = guard
                .iter()
                .map(|(k, _)| *k)
                .filter(|it| *it > new_last_page)
                .collect::<Vec<_>>();
            for pn in keys {
                if let Some(page) = guard.pop(&pn) {
                    Self::evict_cache(file, pn, page)?;
                }
            }
        }
        Ok(())
    }

    pub fn sync(&self, data_only: bool) -> VfsResult<()> {
        let file = self.inner.entry().as_file()?;
        let mut guard = self.page_cache.lock();
        while let Some((pn, page)) = guard.pop_lru() {
            Self::evict_cache(file, pn, page)?;
        }
        file.sync(data_only)?;
        Ok(())
    }

    pub fn location(&self) -> &Location<M> {
        &self.inner
    }
}

impl<M: RawMutex> Drop for CachedFile<M> {
    fn drop(&mut self) {
        if let Err(err) = self.sync(false) {
            warn!("Failed to sync file on drop: {err:?}");
        }
    }
}

/// Low-level interface for file operations.
pub enum FileBackend<M: RawMutex> {
    Cached(Arc<CachedFile<M>>),
    Direct(Location<M>),
}
impl<M: RawMutex> Clone for FileBackend<M> {
    fn clone(&self) -> Self {
        match self {
            Self::Cached(cached) => Self::Cached(Arc::clone(cached)),
            Self::Direct(loc) => Self::Direct(loc.clone()),
        }
    }
}
impl FileBackend<axsync::RawMutex> {
    pub(crate) fn new_cached(location: Location<axsync::RawMutex>) -> Self {
        let key = location.entry().as_ptr();
        let mut guard = CACHED_FILE_TABLE.lock();
        let file = guard.get(&key).unwrap_or_else(|| {
            let file = Arc::new(CachedFile::new(location.clone()));
            guard.insert(key, &file);
            file
        });
        Self::Cached(file)
    }
}
impl<M: RawMutex> FileBackend<M> {
    pub(crate) fn new_direct(location: Location<M>) -> Self {
        Self::Direct(location)
    }

    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> VfsResult<usize> {
        match self {
            Self::Cached(cached) => cached.read_at(buf, offset),
            Self::Direct(loc) => loc.entry().as_file()?.read_at(buf, offset),
        }
    }

    pub fn write_at(&self, buf: &[u8], offset: u64) -> VfsResult<usize> {
        match self {
            Self::Cached(cached) => cached.write_at(buf, offset),
            Self::Direct(loc) => loc.entry().as_file()?.write_at(buf, offset),
        }
    }

    pub fn location(&self) -> &Location<M> {
        match self {
            Self::Cached(cached) => cached.location(),
            Self::Direct(loc) => loc,
        }
    }

    pub fn sync(&self, data_only: bool) -> VfsResult<()> {
        match self {
            Self::Cached(cached) => cached.sync(data_only),
            Self::Direct(loc) => loc.entry().as_file()?.sync(data_only),
        }
    }

    pub fn set_len(&self, len: u64) -> VfsResult<()> {
        match self {
            Self::Cached(cached) => cached.set_len(len),
            Self::Direct(loc) => loc.entry().as_file()?.set_len(len),
        }
    }
}

/// Provides `std::fs::File`-like interface.
pub struct File<M: RawMutex> {
    inner: FileBackend<M>,
    flags: FileFlags,
    position: u64,
}

impl File<axsync::RawMutex> {
    pub fn open(context: &FsContext<axsync::RawMutex>, path: impl AsRef<Path>) -> VfsResult<Self> {
        OpenOptions::new()
            .read(true)
            .open(context, path.as_ref())
            .and_then(OpenResult::into_file)
    }

    pub fn create(
        context: &FsContext<axsync::RawMutex>,
        path: impl AsRef<Path>,
    ) -> VfsResult<Self> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(context, path.as_ref())
            .and_then(OpenResult::into_file)
    }
}
impl<M: RawMutex> File<M> {
    pub(crate) fn new(inner: FileBackend<M>, flags: FileFlags) -> Self {
        Self {
            inner,
            flags,
            position: 0,
        }
    }

    pub fn access(&self, flags: FileFlags) -> VfsResult<&FileBackend<M>> {
        if self.flags.contains(flags) {
            Ok(&self.inner)
        } else {
            Err(VfsError::EPERM)
        }
    }

    pub fn backend(&self) -> &FileBackend<M> {
        &self.inner
    }

    /// Reads a number of bytes starting from a given offset.
    pub fn read_at(&mut self, buf: &mut [u8], offset: u64) -> VfsResult<usize> {
        self.access(FileFlags::READ)?.read_at(buf, offset)
    }

    /// Writes a number of bytes starting from a given offset.
    pub fn write_at(&mut self, buf: &[u8], offset: u64) -> VfsResult<usize> {
        self.access(FileFlags::WRITE)?.write_at(buf, offset)
    }

    /// Attempts to sync OS-internal file content and metadata to disk.
    ///
    /// If `data_only` is `true`, only the file data is synced, not the
    /// metadata.
    pub fn sync(&mut self, data_only: bool) -> VfsResult<()> {
        self.inner.sync(data_only)
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
            let file = self.access(FileFlags::WRITE)?;
            let len = file.location().len()?;
            file.write_at(buf, len).inspect(|n| {
                self.position = len + *n as u64;
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
                let size = self.access(FileFlags::empty())?.location().len()?;
                size.checked_add_signed(off)
                    .ok_or(VfsError::EINVAL)?
                    .clamp(0, size)
            }
            SeekFrom::Current(off) => {
                let size = self.access(FileFlags::empty())?.location().len()?;
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
