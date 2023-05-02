use std::sync::Arc;

use axfs_vfs::{VfsError, VfsNodeType, VfsResult};

use crate::*;

fn test_ramfs_ops(devfs: &RamFileSystem) -> VfsResult {
    const N: usize = 32;
    const N_HALF: usize = N / 2;
    let mut buf = [1; N];

    let root = devfs.root_dir();
    assert!(root.get_attr()?.is_dir());
    assert_eq!(root.get_attr()?.file_type(), VfsNodeType::Dir);
    assert_eq!(
        root.clone().lookup("urandom").err(),
        Some(VfsError::NotFound)
    );
    assert_eq!(
        root.clone().lookup("f1/").err(),
        Some(VfsError::NotADirectory)
    );

    let node = root.lookup("////f1")?;
    assert_eq!(node.get_attr()?.file_type(), VfsNodeType::File);
    assert!(!node.get_attr()?.is_dir());
    assert_eq!(node.get_attr()?.size(), 0);
    assert_eq!(node.read_at(0, &mut buf)?, 0);
    assert_eq!(buf, [1; N]);

    assert_eq!(node.write_at(N_HALF as _, &buf[..N_HALF])?, N_HALF);
    assert_eq!(node.read_at(0, &mut buf)?, N);
    assert_eq!(buf[..N_HALF], [0; N_HALF]);
    assert_eq!(buf[N_HALF..], [1; N_HALF]);
    assert_eq!(node.lookup("/").err(), Some(VfsError::NotADirectory));

    let foo = devfs.root_dir().lookup(".///.//././/.////foo")?;
    assert!(foo.get_attr()?.is_dir());
    assert_eq!(
        foo.read_at(10, &mut buf).err(),
        Some(VfsError::IsADirectory)
    );
    assert!(Arc::ptr_eq(
        &foo.clone().lookup("/f3")?,
        &devfs.root_dir().lookup(".//./foo///f3")?,
    ));
    assert_eq!(
        foo.clone().lookup("/bar//f4")?.get_attr()?.file_type(),
        VfsNodeType::File
    );
    assert_eq!(
        foo.lookup("/bar///")?.get_attr()?.file_type(),
        VfsNodeType::Dir
    );

    Ok(())
}

fn test_get_parent(devfs: &RamFileSystem) -> VfsResult {
    let root = devfs.root_dir();
    assert!(root.parent().is_none());

    let node = root.clone().lookup("f1")?;
    assert!(node.parent().is_none());

    let node = root.clone().lookup(".//foo/bar")?;
    assert!(node.parent().is_some());
    let parent = node.parent().unwrap();
    assert!(Arc::ptr_eq(&parent, &root.clone().lookup("foo")?));
    assert!(parent.lookup("bar").is_ok());

    let node = root.clone().lookup("foo/..")?;
    assert!(Arc::ptr_eq(&node, &root.clone().lookup(".")?));

    assert!(Arc::ptr_eq(
        &root.clone().lookup("/foo/..")?,
        &devfs.root_dir().lookup(".//./foo/././bar/../..")?,
    ));
    assert!(Arc::ptr_eq(
        &root.clone().lookup("././/foo//./../foo//bar///..//././")?,
        &devfs.root_dir().lookup(".//./foo/")?,
    ));
    assert!(Arc::ptr_eq(
        &root.clone().lookup("///foo//bar///../f3")?,
        &root.lookup("foo/.//f3")?,
    ));

    Ok(())
}

#[test]
fn test_ramfs() {
    // .
    // ├── foo
    // │   ├── bar
    // │   │   └── f4
    // │   └── f3
    // ├── f1
    // └── f2

    let ramfs = RamFileSystem::new();
    let root = ramfs.root_dir();
    root.create("f1", VfsNodeType::File).unwrap();
    root.create("f2", VfsNodeType::File).unwrap();
    root.create("foo", VfsNodeType::Dir).unwrap();

    let dir_foo = root.lookup("foo").unwrap();
    dir_foo.create("f3", VfsNodeType::File).unwrap();
    dir_foo.create("bar", VfsNodeType::Dir).unwrap();

    let dir_bar = dir_foo.lookup("bar").unwrap();
    dir_bar.create("f4", VfsNodeType::File).unwrap();

    let mut entries = ramfs.root_dir_node().get_entries();
    entries.sort();
    assert_eq!(entries, ["f1", "f2", "foo"]);

    test_ramfs_ops(&ramfs).unwrap();
    test_get_parent(&ramfs).unwrap();

    let root = ramfs.root_dir();
    assert_eq!(root.remove("f1"), Ok(()));
    assert_eq!(root.remove("//f2"), Ok(()));
    assert_eq!(root.remove("f3").err(), Some(VfsError::NotFound));
    assert_eq!(root.remove("foo").err(), Some(VfsError::DirectoryNotEmpty));
    assert_eq!(root.remove("foo/..").err(), Some(VfsError::InvalidInput));
    assert_eq!(
        root.remove("foo/./bar").err(),
        Some(VfsError::DirectoryNotEmpty)
    );
    assert_eq!(root.remove("foo/bar/f4"), Ok(()));
    assert_eq!(root.remove("foo/bar"), Ok(()));
    assert_eq!(root.remove("./foo//.//f3"), Ok(()));
    assert_eq!(root.remove("./foo"), Ok(()));
    assert!(ramfs.root_dir_node().get_entries().is_empty());
}
