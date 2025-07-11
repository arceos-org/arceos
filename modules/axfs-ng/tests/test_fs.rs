#![allow(unused)]

use std::collections::HashSet;

use axdriver_block::ramdisk::RamDisk;
use axfs_ng::{File, FsContext, fs};
use axfs_ng_vfs::{
    Filesystem, Location, Mountpoint, NodePermission, NodeType, VfsError, VfsResult, path::Path,
};
use axio::Read;

type RawMutex = spin::Mutex<()>;

fn list_files(cx: &FsContext<RawMutex>, path: impl AsRef<Path>) -> VfsResult<HashSet<String>> {
    cx.read_dir(path)?
        .map(|it| it.map(|entry| entry.name.to_owned()))
        .collect()
}

fn test_fs_read(fs: &Filesystem<RawMutex>) -> VfsResult<()> {
    let mount = Mountpoint::new_root(fs);
    let cx: FsContext<spin::mutex::Mutex<()>> = FsContext::new(mount.root_location());

    let names = list_files(&cx, "/").unwrap();
    assert!(
        ["short.txt", "long.txt", "a", "very-long-dir-name"]
            .into_iter()
            .all(|it| names.contains(it))
    );
    assert_eq!(cx.metadata("short.txt")?.size, 14);
    assert_eq!(cx.metadata("long.txt")?.size, 14000);

    let entries = cx.read_dir("/")?.collect::<VfsResult<Vec<_>>>()?;
    for entry in entries {
        assert!(cx.root_dir().lookup_no_follow(&entry.name)?.inode() == entry.ino);
    }

    assert_eq!(
        list_files(&cx, "/a/long/path")?,
        ["test.txt", ".", ".."]
            .into_iter()
            .map(str::to_owned)
            .collect()
    );
    assert_eq!(
        cx.read_to_string("/a/long/path/test.txt")?,
        "Rust is cool!\n"
    );

    assert_eq!(
        cx.resolve("/a/long/path/test.txt")?
            .absolute_path()?
            .to_string(),
        "/a/long/path/test.txt"
    );

    assert!(
        cx.resolve("/very-long-dir-name/very-long-file-name.txt")?
            .is_file()
    );
    let mut file = File::open(&cx, "/very-long-dir-name/very-long-file-name.txt")?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;
    drop(file);
    assert_eq!(core::str::from_utf8(&buf).unwrap(), "Rust is cool!\n");

    Ok(())
}

fn test_fs_write(fs: &Filesystem<RawMutex>) -> VfsResult<()> {
    let mount = Mountpoint::new_root(fs);
    let cx = FsContext::new(mount.root_location());

    let mode = NodePermission::from_bits(0o766).unwrap();
    cx.create_dir("temp", mode)?;
    cx.create_dir("temp2", mode)?;
    assert!(cx.resolve("temp").is_ok() && cx.resolve("temp2").is_ok());
    // cx.rename("temp", "temp2")?;
    // assert!(cx.resolve("temp").is_err() && cx.resolve("temp2").is_ok());

    cx.create_dir("temp", mode)?;
    cx.resolve("temp")?
        .create("test.txt", NodeType::RegularFile, NodePermission::default())?;
    assert!(matches!(
        cx.rename("temp2", "temp"),
        Err(VfsError::ENOTEMPTY)
    ));

    cx.write("/test.txt", "hello world".as_bytes())?;
    assert_eq!(cx.read_to_string("/test.txt")?, "hello world");

    cx.create_dir("test_dir", NodePermission::from_bits_truncate(0o755))?;
    cx.rename("test_dir", "test")?;
    cx.remove_dir("test")?;

    println!("---------------------");

    if cx.link("/test.txt", "/test_link").is_ok() {
        assert_eq!(cx.read_to_string("/test_link")?, "hello world");
    }
    if cx.symlink("/test.txt", "/test_symlink").is_ok() {
        assert_eq!(cx.read_to_string("/test_symlink")?, "hello world");
    }

    // FAT has errornous rename implementation
    if fs.name() != "vfat" {
        cx.write("rename1", "hello world".as_bytes())?;
        cx.write("rename2", "hello world2".as_bytes())?;
        cx.rename("rename1", "rename2")?;
        assert_eq!(cx.read_to_string("rename2")?, "hello world");
    }

    Ok(())
}

fn test_fs_full(fs: Filesystem<RawMutex>) -> VfsResult<()> {
    let mut thrds = vec![];
    for _ in 0..1 {
        let fs = fs.clone();
        thrds.push(std::thread::spawn(move || test_fs_read(&fs)));
    }
    for th in thrds {
        th.join().unwrap()?;
    }
    test_fs_write(&fs)?;
    Ok(())
}

#[test]
#[cfg(feature = "fat")]
fn test_fatfs() {
    for path in ["resources/fat16.img", "resources/fat32.img"] {
        let data = std::fs::read(path).unwrap();
        let disk = RamDisk::from(&data);
        let fs = fs::fat::FatFilesystem::<RawMutex>::new(disk);
        test_fs_full(fs).unwrap();
    }
}

#[test]
#[cfg(feature = "ext4")]
fn test_ext4() {
    let data = std::fs::read("resources/ext4.img").unwrap();
    let disk = RamDisk::from(&data);
    let fs = fs::ext4::Ext4Filesystem::<RawMutex>::new(disk).unwrap();
    test_fs_full(fs).unwrap();
}

#[test]
#[cfg(all(feature = "ext4", feature = "fat"))]
fn test_mount() {
    env_logger::init();
    let disk = RamDisk::from(&std::fs::read("resources/ext4.img").unwrap());
    let fs = fs::ext4::Ext4Filesystem::<RawMutex>::new(disk).unwrap();

    let disk = RamDisk::from(&std::fs::read("resources/fat16.img").unwrap());
    let sub_fs = fs::fat::FatFilesystem::<RawMutex>::new(disk);

    let mount = Mountpoint::new(&fs, None);
    let cx = FsContext::new(mount.root_location());
    cx.resolve("a").unwrap().mount(&sub_fs);

    let mt = cx.resolve("a").unwrap();
    assert!(!mt.is_mountpoint() && mt.is_root_of_mount());
    assert_eq!(mt.filesystem().name(), "vfat");
    assert_eq!(mt.absolute_path().unwrap().to_string(), "/a");

    assert_eq!(
        cx.read_to_string("/a/../a/very-long-dir-name/very-long-file-name.txt")
            .unwrap(),
        "Rust is cool!\n"
    );
}
