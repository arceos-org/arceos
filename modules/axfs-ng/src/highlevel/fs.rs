use alloc::{
    borrow::{Cow, ToOwned},
    collections::vec_deque::VecDeque,
    string::String,
    sync::Arc,
    vec::Vec,
};

use axfs_ng_vfs::{
    Location, Metadata, NodePermission, NodeType, VfsError, VfsResult,
    path::{Component, Components, Path, PathBuf},
};
use axio::{Read, Write};
use axsync::Mutex;
use spin::Once;

use super::File;

pub const SYMLINKS_MAX: usize = 40;

pub static ROOT_FS_CONTEXT: Once<FsContext> = Once::new();

scope_local::scope_local! {
    pub static FS_CONTEXT: Arc<Mutex<FsContext>> =
        Arc::new(Mutex::new(
            ROOT_FS_CONTEXT
                .get()
                .expect("Root FS context not initialized")
                .clone(),
        ));
}

pub struct ReadDirEntry {
    pub name: String,
    pub ino: u64,
    pub node_type: NodeType,
    pub offset: u64,
}

/// Provides `std::fs`-like interface.
#[derive(Debug, Clone)]
pub struct FsContext {
    root_dir: Location,
    current_dir: Location,
}

impl FsContext {
    pub fn new(root_dir: Location) -> Self {
        Self {
            root_dir: root_dir.clone(),
            current_dir: root_dir,
        }
    }

    pub fn root_dir(&self) -> &Location {
        &self.root_dir
    }

    pub fn current_dir(&self) -> &Location {
        &self.current_dir
    }

    pub fn set_current_dir(&mut self, current_dir: Location) -> VfsResult<()> {
        current_dir.check_is_dir()?;
        self.current_dir = current_dir;
        Ok(())
    }

    pub fn with_current_dir(&self, current_dir: Location) -> VfsResult<Self> {
        current_dir.check_is_dir()?;
        Ok(Self {
            root_dir: self.root_dir.clone(),
            current_dir,
        })
    }

    /// Attempts to resolve a possible symlink, at the current location (this
    /// assumes that `loc` is a child of current directory).
    pub fn try_resolve_symlink(
        &self,
        loc: Location,
        follow_count: &mut usize,
    ) -> VfsResult<Location> {
        if loc.node_type() != NodeType::Symlink {
            return Ok(loc);
        }
        if *follow_count >= SYMLINKS_MAX {
            return Err(VfsError::FilesystemLoop);
        }
        *follow_count += 1;
        let target = loc.read_link()?;
        if target.is_empty() {
            return Err(VfsError::NotFound);
        }
        self.resolve_components(PathBuf::from(target).components(), follow_count)
    }

    fn lookup(&self, dir: &Location, name: &str, follow_count: &mut usize) -> VfsResult<Location> {
        let loc = dir.lookup_no_follow(name)?;
        self.with_current_dir(dir.clone())?
            .try_resolve_symlink(loc, follow_count)
    }

    fn resolve_components(
        &self,
        components: Components,
        follow_count: &mut usize,
    ) -> VfsResult<Location> {
        let mut dir = self.current_dir.clone();
        for comp in components {
            match comp {
                Component::CurDir => {}
                Component::ParentDir => {
                    dir = dir.parent().unwrap_or_else(|| self.root_dir.clone());
                }
                Component::RootDir => {
                    dir = self.root_dir.clone();
                }
                Component::Normal(name) => {
                    dir = self.lookup(&dir, name, follow_count)?;
                }
            }
        }
        Ok(dir)
    }

    fn resolve_inner<'a>(
        &self,
        path: &'a Path,
        follow_count: &mut usize,
    ) -> VfsResult<(Location, Option<&'a str>)> {
        let entry_name = path.file_name();
        let mut components = path.components();
        if entry_name.is_some() {
            components.next_back();
        }
        let dir = self.resolve_components(components, follow_count)?;
        dir.check_is_dir()?;
        Ok((dir, entry_name))
    }

    /// Resolves a path starting from `current_dir`.
    pub fn resolve(&self, path: impl AsRef<Path>) -> VfsResult<Location> {
        let mut follow_count = 0;
        let (dir, name) = self.resolve_inner(path.as_ref(), &mut follow_count)?;
        match name {
            Some(name) => self.lookup(&dir, name, &mut follow_count),
            None => Ok(dir),
        }
    }

    /// Resolves a path starting from `current_dir` not following symlinks.
    pub fn resolve_no_follow(&self, path: impl AsRef<Path>) -> VfsResult<Location> {
        let (dir, name) = self.resolve_inner(path.as_ref(), &mut 0)?;
        match name {
            Some(name) => dir.lookup_no_follow(name),
            None => Ok(dir),
        }
    }

    /// Taking current node as root directory, resolves a path starting from
    /// `current_dir`.
    ///
    /// Returns `(parent_dir, entry_name)`, where `entry_name` is the name of
    /// the entry.
    pub fn resolve_parent<'a>(&self, path: &'a Path) -> VfsResult<(Location, Cow<'a, str>)> {
        let (dir, name) = self.resolve_inner(path, &mut 0)?;
        if let Some(name) = name {
            Ok((dir, Cow::Borrowed(name)))
        } else if let Some(parent) = dir.parent() {
            Ok((parent, Cow::Owned(dir.name().to_owned())))
        } else {
            Err(VfsError::InvalidInput)
        }
    }

    /// Resolves a path starting from `current_dir`, returning the parent
    /// directory and the name of the entry.
    ///
    /// This function requires that the entry does not exist and the parent
    /// exists. Note that, it does not perform an actual check to ensure the
    /// entry's non-existence. It simply raises an error if the entry name is
    /// not present in the path.
    pub fn resolve_nonexistent<'a>(&self, path: &'a Path) -> VfsResult<(Location, &'a str)> {
        let (dir, name) = self.resolve_inner(path, &mut 0)?;
        if let Some(name) = name {
            Ok((dir, name))
        } else {
            Err(VfsError::InvalidInput)
        }
    }

    /// Retrieves metadata for the file.
    pub fn metadata(&self, path: impl AsRef<Path>) -> VfsResult<Metadata> {
        self.resolve(path)?.metadata()
    }

    /// Reads the entire contents of a file into a bytes vector.
    pub fn read(&self, path: impl AsRef<Path>) -> VfsResult<Vec<u8>> {
        let mut buf = Vec::new();
        let file = File::open(self, path.as_ref())?;
        (&file).read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Reads the entire contents of a file into a string.
    pub fn read_to_string(&self, path: impl AsRef<Path>) -> VfsResult<String> {
        String::from_utf8(self.read(path)?).map_err(|_| VfsError::InvalidData)
    }

    /// Writes a slice as the entire contents of a file.
    ///
    /// This function will create a file if it does not exist, and will entirely
    /// replace its contents if it does.
    pub fn write(&self, path: impl AsRef<Path>, buf: impl AsRef<[u8]>) -> VfsResult<()> {
        let file = File::create(self, path.as_ref())?;
        (&file).write_all(buf.as_ref())?;
        Ok(())
    }

    /// Returns an iterator over the entries in a directory.
    pub fn read_dir(&self, path: impl AsRef<Path>) -> VfsResult<ReadDir> {
        let dir = self.resolve(path)?;
        Ok(ReadDir {
            dir,
            buf: VecDeque::new(),
            offset: 0,
            ended: false,
        })
    }

    /// Removes a file from the filesystem.
    pub fn remove_file(&self, path: impl AsRef<Path>) -> VfsResult<()> {
        let entry = self.resolve_no_follow(path.as_ref())?;
        entry
            .parent()
            .ok_or(VfsError::IsADirectory)?
            .unlink(entry.name(), false)
    }

    /// Removes a directory from the filesystem.
    pub fn remove_dir(&self, path: impl AsRef<Path>) -> VfsResult<()> {
        let entry = self.resolve_no_follow(path.as_ref())?;
        entry
            .parent()
            .ok_or(VfsError::ResourceBusy)?
            .unlink(entry.name(), true)
    }

    /// Renames a file or directory to a new name, replacing the original file
    /// if `to` already exists.
    pub fn rename(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> VfsResult<()> {
        let (src_dir, src_name) = self.resolve_parent(from.as_ref())?;
        let (dst_dir, dst_name) = self.resolve_parent(to.as_ref())?;
        src_dir.rename(&src_name, &dst_dir, &dst_name)
    }

    /// Creates a new, empty directory at the provided path.
    pub fn create_dir(&self, path: impl AsRef<Path>, mode: NodePermission) -> VfsResult<Location> {
        let (dir, name) = self.resolve_nonexistent(path.as_ref())?;
        dir.create(name, NodeType::Directory, mode)
    }

    /// Creates a new hard link on the filesystem.
    pub fn link(
        &self,
        old_path: impl AsRef<Path>,
        new_path: impl AsRef<Path>,
    ) -> VfsResult<Location> {
        let old = self.resolve(old_path.as_ref())?;
        let (new_dir, new_name) = self.resolve_nonexistent(new_path.as_ref())?;
        new_dir.link(new_name, &old)
    }

    /// Creates a new symbolic link on the filesystem.
    pub fn symlink(
        &self,
        target: impl AsRef<str>,
        link_path: impl AsRef<Path>,
    ) -> VfsResult<Location> {
        let (dir, name) = self.resolve_nonexistent(link_path.as_ref())?;
        if dir.lookup_no_follow(name).is_ok() {
            return Err(VfsError::AlreadyExists);
        }
        let symlink = dir.create(name, NodeType::Symlink, NodePermission::default())?;
        symlink.entry().as_file()?.set_symlink(target.as_ref())?;
        Ok(symlink)
    }

    /// Returns the canonical, absolute form of a path.
    pub fn canonicalize(&self, path: impl AsRef<Path>) -> VfsResult<PathBuf> {
        self.resolve(path.as_ref())?.absolute_path()
    }
}

/// Iterator returned by [`FsContext::read_dir`].
pub struct ReadDir {
    dir: Location,
    buf: VecDeque<ReadDirEntry>,
    offset: u64,
    ended: bool,
}

impl ReadDir {
    // TODO: tune this
    pub const BUF_SIZE: usize = 128;
}

impl Iterator for ReadDir {
    type Item = VfsResult<ReadDirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }

        if self.buf.is_empty() {
            self.buf.clear();
            let result = self.dir.read_dir(
                self.offset,
                &mut |name: &str, ino: u64, node_type: NodeType, offset: u64| {
                    self.buf.push_back(ReadDirEntry {
                        name: name.to_owned(),
                        ino,
                        node_type,
                        offset,
                    });
                    self.offset = offset;
                    self.buf.len() < Self::BUF_SIZE
                },
            );

            // We handle errors only if we didn't get any entries
            if self.buf.is_empty() {
                if let Err(err) = result {
                    return Some(Err(err));
                }
                self.ended = true;
                return None;
            }
        }

        self.buf.pop_front().map(Ok)
    }
}
