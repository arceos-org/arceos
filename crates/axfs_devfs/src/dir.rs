use alloc::{collections::BTreeMap, sync::Arc};
use axfs_vfs::{VfsDirEntry, VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType};
use axfs_vfs::{VfsError, VfsResult};

#[derive(Default)]
pub struct DirNode {
    children: BTreeMap<&'static str, VfsNodeRef>,
}

impl DirNode {
    pub const fn new() -> Self {
        Self {
            children: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, name: &'static str, node: VfsNodeRef) {
        self.children.insert(name, node);
    }
}

impl VfsNodeOps for DirNode {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new_dir(4096, 0))
    }

    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        // TODO: parent
        let (name, rest) = split_path(path);
        let node = if name == "." || name.is_empty() {
            self.clone()
        } else {
            self.children.get(name).ok_or(VfsError::NotFound)?.clone()
        };

        if let Some(rest) = rest {
            node.lookup(rest)
        } else {
            Ok(node)
        }
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        let mut children = self.children.iter().skip(start_idx.max(2) - 2);
        for (i, ent) in dirents.iter_mut().enumerate() {
            match i + start_idx {
                0 => *ent = VfsDirEntry::new(".", VfsNodeType::Dir),
                1 => *ent = VfsDirEntry::new("..", VfsNodeType::Dir),
                _ => {
                    if let Some((name, node)) = children.next() {
                        *ent = VfsDirEntry::new(name, node.get_attr().unwrap().file_type());
                    } else {
                        return Ok(i);
                    }
                }
            }
        }
        Ok(dirents.len())
    }

    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        log::debug!("create {:?} at devfs: {}", ty, path);
        Err(VfsError::PermissionDenied) // do not support to create nodes dynamically
    }

    fn remove(&self, path: &str) -> VfsResult {
        log::debug!("remove at devfs: {}", path);
        Err(VfsError::PermissionDenied) // do not support to remove nodes dynamically
    }

    axfs_vfs::impl_vfs_dir_default! {}
}

fn split_path(path: &str) -> (&str, Option<&str>) {
    let trimmed_path = path.trim_start_matches('/');
    trimmed_path.find('/').map_or((trimmed_path, None), |n| {
        (&trimmed_path[..n], Some(&trimmed_path[n + 1..]))
    })
}
