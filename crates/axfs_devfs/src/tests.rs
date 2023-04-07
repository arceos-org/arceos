use std::sync::Arc;

use axfs_vfs::{VfsError, VfsNodeType, VfsResult};

use crate::*;

fn test_devfs_ops(devfs: &DeviceFileSystem) -> VfsResult {
    const N: usize = 32;
    let mut buf = [1; N];

    let root = devfs.root_dir();
    assert!(root.get_attr()?.is_dir());
    assert_eq!(root.get_attr()?.file_type(), VfsNodeType::Dir);
    assert_eq!(
        root.clone().lookup("urandom").err(),
        Some(VfsError::NotFound)
    );
    assert_eq!(
        root.clone().lookup("zero/").err(),
        Some(VfsError::NotADirectory)
    );

    let node = root.lookup("////null")?;
    assert_eq!(node.get_attr()?.file_type(), VfsNodeType::CharDevice);
    assert!(!node.get_attr()?.is_dir());
    assert_eq!(node.get_attr()?.size(), 0);
    assert_eq!(node.read_at(0, &mut buf)?, 0);
    assert_eq!(buf, [1; N]);
    assert_eq!(node.write_at(N as _, &buf)?, N);
    assert_eq!(node.lookup("/").err(), Some(VfsError::NotADirectory));

    let node = devfs.root_dir().lookup(".///.//././/.////zero")?;
    assert_eq!(node.get_attr()?.file_type(), VfsNodeType::CharDevice);
    assert!(!node.get_attr()?.is_dir());
    assert_eq!(node.get_attr()?.size(), 0);
    assert_eq!(node.read_at(10, &mut buf)?, N);
    assert_eq!(buf, [0; N]);
    assert_eq!(node.write_at(0, &buf)?, N);

    let foo = devfs.root_dir().lookup(".///.//././/.////foo")?;
    assert!(foo.get_attr()?.is_dir());
    assert_eq!(
        foo.read_at(10, &mut buf).err(),
        Some(VfsError::IsADirectory)
    );
    assert!(Arc::ptr_eq(
        &foo.clone().lookup("/f2")?,
        &devfs.root_dir().lookup(".//./foo///f2")?,
    ));
    assert_eq!(
        foo.clone().lookup("/bar//f1")?.get_attr()?.file_type(),
        VfsNodeType::CharDevice
    );
    assert_eq!(
        foo.lookup("/bar///")?.get_attr()?.file_type(),
        VfsNodeType::Dir
    );

    Ok(())
}

fn test_get_parent(devfs: &DeviceFileSystem) -> VfsResult {
    let root = devfs.root_dir();
    assert!(root.parent().is_none());

    let node = root.clone().lookup("null")?;
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
        &root.clone().lookup("///foo//bar///../f2")?,
        &root.lookup("foo/.//f2")?,
    ));

    Ok(())
}

#[test]
fn test_devfs() {
    // .
    // ├── foo
    // │   ├── bar
    // │   │   └── f1 (null)
    // │   └── f2 (zero)
    // ├── null
    // └── zero

    let devfs = DeviceFileSystem::new();
    devfs.add("null", Arc::new(NullDev));
    devfs.add("zero", Arc::new(ZeroDev));

    let dir_foo = devfs.mkdir("foo");
    dir_foo.add("f2", Arc::new(ZeroDev));
    let dir_bar = dir_foo.mkdir("bar");
    dir_bar.add("f1", Arc::new(NullDev));

    test_devfs_ops(&devfs).unwrap();
    test_get_parent(&devfs).unwrap();
}
