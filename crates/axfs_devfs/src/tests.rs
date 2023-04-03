use std::sync::Arc;

use axfs_vfs::{VfsError, VfsNodeType, VfsResult};

use crate::*;

fn test_devfs_ops(devfs: DeviceFileSystem) -> VfsResult {
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

#[test]
fn test_devfs() {
    // .
    // ├── foo
    // │   ├── bar
    // │   │   └── f1 (null)
    // │   └── f2 (zero)
    // ├── null
    // └── zero

    let mut devfs = DeviceFileSystem::new();
    devfs.add("null", Arc::new(NullDev));
    devfs.add("zero", Arc::new(ZeroDev));

    let mut dir_foo = DirNode::new();
    let mut dir_bar = DirNode::new();
    dir_bar.add("f1", Arc::new(NullDev));
    dir_foo.add("bar", Arc::new(dir_bar));
    dir_foo.add("f2", Arc::new(ZeroDev));
    devfs.add("foo", Arc::new(dir_foo));

    test_devfs_ops(devfs).unwrap();
}
