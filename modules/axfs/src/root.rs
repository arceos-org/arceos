// Copyright 2025 The Axvisor Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Root directory of the filesystem
//!
//! TODO: it doesn't work very well if the mount points have containment relationships.

use alloc::{
    borrow::ToOwned,
    collections::BTreeMap,
    format,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use axerrno::{AxError, AxResult, ax_err};
use axfs_vfs::{VfsDirEntry, VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps, VfsResult};
use lazyinit::LazyInit;
use spin::Mutex;

use crate::{
    api::FileType,
    mounts,
    partition::{FilesystemType, PartitionInfo, create_filesystem_for_partition},
};

static CURRENT_DIR_PATH: Mutex<String> = Mutex::new(String::new());
static CURRENT_DIR: LazyInit<Mutex<VfsNodeRef>> = LazyInit::new();

struct MountPoint {
    path: String,
    fs: Arc<dyn VfsOps>,
}

pub struct RootDirectory {
    main_fs: Arc<dyn VfsOps>,
    mounts: Vec<MountPoint>,
}

static ROOT_DIR: LazyInit<Arc<RootDirectory>> = LazyInit::new();

impl MountPoint {
    pub fn new(path: String, fs: Arc<dyn VfsOps>) -> Self {
        Self { path, fs }
    }
}

impl Drop for MountPoint {
    fn drop(&mut self) {
        self.fs.umount().ok();
    }
}

impl RootDirectory {
    pub const fn new(main_fs: Arc<dyn VfsOps>) -> Self {
        Self {
            main_fs,
            mounts: Vec::new(),
        }
    }

    pub fn mount(&mut self, path: &str, fs: Arc<dyn VfsOps>) -> AxResult {
        if path == "/" {
            return ax_err!(InvalidInput, "cannot mount root filesystem");
        }
        if !path.starts_with('/') {
            return ax_err!(InvalidInput, "mount path must start with '/'");
        }
        if self.mounts.iter().any(|mp| mp.path == path) {
            return ax_err!(InvalidInput, "mount point already exists");
        }
        // create the mount point in the main filesystem if it does not exist
        self.main_fs.root_dir().create(path, FileType::Dir)?;
        fs.mount(path, self.main_fs.root_dir().lookup(path)?)?;
        self.mounts.push(MountPoint::new(path.to_owned(), fs));
        Ok(())
    }

    pub fn _umount(&mut self, path: &str) {
        self.mounts.retain(|mp| mp.path != path);
    }

    pub fn contains(&self, path: &str) -> bool {
        self.mounts.iter().any(|mp| mp.path == path)
    }

    fn lookup_mounted_fs<F, T>(&self, path: &str, f: F) -> AxResult<T>
    where
        F: FnOnce(Arc<dyn VfsOps>, &str) -> AxResult<T>,
    {
        debug!("lookup at root: {}", path);
        let normalized_path = self.normalize_path(path);

        // Find the best matching mount point
        if let Some((mount_fs, rest_path)) = self.find_best_mount(&normalized_path) {
            f(mount_fs, rest_path)
        } else {
            // No mount point matched, use main filesystem
            f(self.main_fs.clone(), &normalized_path)
        }
    }

    /// Normalize path by trimming leading '/' and handling './' prefix
    fn normalize_path<'a>(&self, path: &'a str) -> &'a str {
        let path = path.trim_matches('/');
        if let Some(rest) = path.strip_prefix("./") {
            rest
        } else {
            path
        }
    }

    /// Find the best matching mount point for the given path
    /// Returns (filesystem, remaining_path) if a match is found
    fn find_best_mount<'a>(&self, path: &'a str) -> Option<(Arc<dyn VfsOps>, &'a str)> {
        let mut best_match = None;
        let mut max_len = 0;

        for (i, mp) in self.mounts.iter().enumerate() {
            // Skip the first '/' in mount path for comparison
            let mount_path = &mp.path[1..];

            if path.starts_with(mount_path) && mp.path.len() - 1 > max_len {
                max_len = mp.path.len() - 1;
                best_match = Some(i);
            }
        }

        if let Some(idx) = best_match {
            let rest_path = &path[max_len..];
            Some((self.mounts[idx].fs.clone(), rest_path))
        } else {
            None
        }
    }
}

impl VfsNodeOps for RootDirectory {
    axfs_vfs::impl_vfs_dir_default! {}

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        self.main_fs.root_dir().get_attr()
    }

    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        self.lookup_mounted_fs(path, |fs, rest_path| fs.root_dir().lookup(rest_path))
    }

    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        self.lookup_mounted_fs(path, |fs, rest_path| {
            if rest_path.is_empty() {
                Ok(()) // already exists
            } else {
                fs.root_dir().create(rest_path, ty)
            }
        })
    }

    fn remove(&self, path: &str) -> VfsResult {
        self.lookup_mounted_fs(path, |fs, rest_path| {
            if rest_path.is_empty() {
                ax_err!(PermissionDenied) // cannot remove mount points
            } else {
                fs.root_dir().remove(rest_path)
            }
        })
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        let mut all_entries = Vec::new();

        // Add mount points
        for mp in &self.mounts {
            let name = &mp.path[1..];
            all_entries.push((name.to_string(), VfsNodeType::Dir));
        }

        // Add from main_fs
        let mut main_dirents = Vec::with_capacity(64);
        for _ in 0..64 {
            main_dirents.push(VfsDirEntry::default());
        }
        let main_count = self.main_fs.root_dir().read_dir(0, &mut main_dirents)?;
        for i in 0..main_count {
            let name_bytes = main_dirents[i].name_as_bytes();
            if let Ok(name_str) = core::str::from_utf8(name_bytes) {
                if !name_str.is_empty() {
                    let ty = main_dirents[i].entry_type();
                    all_entries.push((name_str.to_string(), ty));
                }
            }
        }

        // Unique
        let mut unique = BTreeMap::new();
        for (name, ty) in all_entries {
            unique.insert(name, ty);
        }

        let unique_vec: Vec<_> = unique.into_iter().collect();
        let mut count = 0;
        for (name, ty) in unique_vec.iter().skip(start_idx) {
            if count >= dirents.len() {
                break;
            }
            dirents[count] = VfsDirEntry::new(name, *ty);
            count += 1;
        }
        Ok(count)
    }
}

pub(crate) fn init_rootfs_with_ramfs() {
    info!("Initializing root filesystem with ramfs");
    let main_fs = mounts::ramfs();
    let root_dir = RootDirectory::new(main_fs);
    mount_virtual_fs(root_dir);
}

/// Find and create root filesystem from partitions
fn find_root_filesystem(
    disk: &Arc<crate::dev::Disk>,
    partitions: &[PartitionInfo],
    root_partition_index: Option<usize>,
) -> (Option<Arc<dyn VfsOps>>, Option<usize>) {
    // Try to use the specified partition index first
    if let Some(index) = root_partition_index {
        if let Some((fs, idx)) = try_use_specified_partition(disk, partitions, index) {
            return (Some(fs), Some(idx));
        }
    }

    // Fall back to first partition with supported filesystem
    if let Some((fs, idx)) = find_first_supported_partition(disk, partitions) {
        return (Some(fs), Some(idx));
    }

    (None, None)
}

/// Try to use the specified partition as root filesystem
fn try_use_specified_partition(
    disk: &Arc<crate::dev::Disk>,
    partitions: &[PartitionInfo],
    index: usize,
) -> Option<(Arc<dyn VfsOps>, usize)> {
    if index >= partitions.len() {
        warn!(
            "Specified partition index {} is out of range (total partitions: {})",
            index,
            partitions.len()
        );
        return None;
    }

    let partition = &partitions[index];
    if partition.filesystem_type.is_none() {
        warn!(
            "Specified partition '{}' has no supported filesystem",
            partition.name
        );
        return None;
    }

    match create_filesystem_for_partition((**disk).clone(), partition) {
        Ok(fs) => {
            info!(
                "Using specified partition '{}' ({:?}) as root filesystem",
                partition.name,
                partition.filesystem_type.unwrap_or(FilesystemType::Unknown)
            );
            Some((fs, index))
        }
        Err(e) => {
            warn!(
                "Failed to create filesystem for specified partition '{}': {:?}",
                partition.name, e
            );
            None
        }
    }
}

/// Find the first partition with a supported filesystem
fn find_first_supported_partition(
    disk: &Arc<crate::dev::Disk>,
    partitions: &[PartitionInfo],
) -> Option<(Arc<dyn VfsOps>, usize)> {
    for (i, partition) in partitions.iter().enumerate() {
        if partition.filesystem_type.is_some() {
            match create_filesystem_for_partition((**disk).clone(), partition) {
                Ok(fs) => {
                    info!(
                        "Using partition '{}' ({:?}) as root filesystem",
                        partition.name,
                        partition.filesystem_type.unwrap_or(FilesystemType::Unknown)
                    );
                    return Some((fs, i));
                }
                Err(e) => {
                    warn!(
                        "Failed to create filesystem for partition '{}': {:?}",
                        partition.name, e
                    );
                }
            }
        }
    }
    None
}

/// Mount additional partitions (non-root partitions)
fn mount_additional_partitions(
    disk: &Arc<crate::dev::Disk>,
    root_dir: &mut RootDirectory,
    partitions: &[PartitionInfo],
    root_partition_index: Option<usize>,
) {
    // Create /boot directory first if it doesn't exist
    if let Err(e) = root_dir.main_fs.root_dir().create("/boot", FileType::Dir) {
        warn!("Failed to create /boot directory: {:?}", e);
    }

    // Mount all non-root partitions
    for (i, partition) in partitions.iter().enumerate() {
        // Skip root partition
        if Some(i) == root_partition_index {
            continue;
        }

        // Only mount partitions with supported filesystems
        if partition.filesystem_type.is_some() {
            mount_single_partition(disk, root_dir, partition);
        }
    }
}

/// Mount a single partition
fn mount_single_partition(
    disk: &Arc<crate::dev::Disk>,
    root_dir: &mut RootDirectory,
    partition: &PartitionInfo,
) {
    match create_filesystem_for_partition((**disk).clone(), partition) {
        Ok(fs) => {
            // Determine mount path based on partition name
            let mount_path = if partition.name.to_lowercase().contains("boot") {
                String::from("/boot")
            } else {
                format!("/{}", partition.name)
            };

            info!(
                "Mounting partition '{}' at '{}'",
                partition.name, mount_path
            );

            // Create mount point directory in root filesystem
            if let Err(e) = root_dir
                .main_fs
                .root_dir()
                .create(&mount_path, FileType::Dir)
            {
                warn!("Failed to create mount point '{}': {:?}", mount_path, e);
                return;
            }

            // Mount filesystem
            if let Err(e) = root_dir.mount(&mount_path, fs) {
                warn!(
                    "Failed to mount partition '{}' at '{}': {:?}",
                    partition.name, mount_path, e
                );
            }
        }
        Err(e) => {
            warn!(
                "Failed to create filesystem for partition '{}': {:?}",
                partition.name, e
            );
        }
    }
}

/// Initialize root filesystem with dynamic partition detection and specified root partition
pub(crate) fn init_rootfs_with_partitions(
    disk: Arc<crate::dev::Disk>,
    partitions: Vec<PartitionInfo>,
    root_partition_index: Option<usize>,
) -> bool {
    info!(
        "Initializing root filesystem with {} partitions",
        partitions.len()
    );

    // Find and create the root filesystem
    let (main_fs, actual_root_partition_index) =
        find_root_filesystem(&disk, &partitions, root_partition_index);

    // If no supported filesystem found, fall back to ramfs
    let main_fs = match main_fs {
        Some(fs) => fs,
        None => {
            warn!("No supported filesystem found in partitions, mount ramfs as rootfs");
            mounts::ramfs()
        }
    };

    let mut root_dir = RootDirectory::new(main_fs);

    // Mount additional partitions
    mount_additional_partitions(
        &disk,
        &mut root_dir,
        &partitions,
        actual_root_partition_index,
    );

    mount_virtual_fs(root_dir);
    true
}

pub fn mount_virtual_fs(mut root_dir: RootDirectory) {
    // Mount virtual filesystems
    if let Err(e) = root_dir
        .mount("/proc", mounts::procfs().unwrap())
        .and_then(|_| root_dir.mount("/sys", mounts::sysfs().unwrap()))
    {
        panic!("Failed to mount virtual filesystems: {:?}", e);
    }

    // Initialize global state
    let root_dir = Arc::new(root_dir);
    ROOT_DIR.init_once(root_dir.clone());
    CURRENT_DIR.init_once(Mutex::new(ROOT_DIR.clone()));
    *CURRENT_DIR_PATH.lock() = "/".into();
}

fn parent_node_of(dir: Option<&VfsNodeRef>, path: &str) -> VfsNodeRef {
    if path.starts_with('/') {
        ROOT_DIR.clone()
    } else {
        dir.cloned().unwrap_or_else(|| CURRENT_DIR.lock().clone())
    }
}

pub(crate) fn absolute_path(path: &str) -> AxResult<String> {
    if path.starts_with('/') {
        Ok(axfs_vfs::path::canonicalize(path))
    } else {
        let path = CURRENT_DIR_PATH.lock().clone() + path;
        Ok(axfs_vfs::path::canonicalize(&path))
    }
}

pub(crate) fn lookup(dir: Option<&VfsNodeRef>, path: &str) -> AxResult<VfsNodeRef> {
    if path.is_empty() {
        return ax_err!(NotFound);
    }
    let node = parent_node_of(dir, path).lookup(path)?;
    if path.ends_with('/') && !node.get_attr()?.is_dir() {
        ax_err!(NotADirectory)
    } else {
        Ok(node)
    }
}

pub(crate) fn create_file(dir: Option<&VfsNodeRef>, path: &str) -> AxResult<VfsNodeRef> {
    if path.is_empty() {
        return ax_err!(NotFound);
    } else if path.ends_with('/') {
        return ax_err!(NotADirectory);
    }
    let parent = parent_node_of(dir, path);
    parent.create(path, VfsNodeType::File)?;
    parent.lookup(path)
}

pub(crate) fn create_dir(dir: Option<&VfsNodeRef>, path: &str) -> AxResult {
    match lookup(dir, path) {
        Ok(_) => ax_err!(AlreadyExists),
        Err(AxError::NotFound) => parent_node_of(dir, path).create(path, VfsNodeType::Dir),
        Err(e) => Err(e),
    }
}

pub(crate) fn remove_file(dir: Option<&VfsNodeRef>, path: &str) -> AxResult {
    let node = lookup(dir, path)?;
    let attr = node.get_attr()?;
    if attr.is_dir() {
        ax_err!(IsADirectory)
    } else if !attr.perm().owner_writable() {
        ax_err!(PermissionDenied)
    } else {
        parent_node_of(dir, path).remove(path)
    }
}

pub(crate) fn remove_dir(dir: Option<&VfsNodeRef>, path: &str) -> AxResult {
    if path.is_empty() {
        return ax_err!(NotFound);
    }
    let path_check = path.trim_matches('/');
    if path_check.is_empty() {
        return ax_err!(DirectoryNotEmpty); // rm -d '/'
    } else if path_check == "."
        || path_check == ".."
        || path_check.ends_with("/.")
        || path_check.ends_with("/..")
    {
        return ax_err!(InvalidInput);
    }
    if ROOT_DIR.contains(&absolute_path(path)?) {
        return ax_err!(PermissionDenied);
    }

    let node = lookup(dir, path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        ax_err!(NotADirectory)
    } else if !attr.perm().owner_writable() {
        ax_err!(PermissionDenied)
    } else {
        parent_node_of(dir, path).remove(path)
    }
}

pub(crate) fn current_dir() -> AxResult<String> {
    Ok(CURRENT_DIR_PATH.lock().clone())
}

pub(crate) fn set_current_dir(path: &str) -> AxResult {
    let mut abs_path = absolute_path(path)?;
    if !abs_path.ends_with('/') {
        abs_path += "/";
    }
    if abs_path == "/" {
        *CURRENT_DIR.lock() = ROOT_DIR.clone();
        *CURRENT_DIR_PATH.lock() = "/".into();
        return Ok(());
    }

    let node = lookup(None, &abs_path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        ax_err!(NotADirectory)
    } else if !attr.perm().owner_executable() {
        ax_err!(PermissionDenied)
    } else {
        *CURRENT_DIR.lock() = node;
        *CURRENT_DIR_PATH.lock() = abs_path;
        Ok(())
    }
}

pub(crate) fn rename(old: &str, new: &str) -> AxResult {
    if parent_node_of(None, new).lookup(new).is_ok() {
        warn!("dst file already exist, now remove it");
        remove_file(None, new)?;
    }
    parent_node_of(None, old).rename(old, new)
}
