#![cfg(not(feature = "use-virtio-blk"))]

use axfs::api as fs;
use axio as io;

use driver_block::ramdisk::RamDisk;
use fs::{File, FileType, OpenOptions};
use io::{prelude::*, Error, Result};

const IMG_PATH: &str = "resources/fat16.img";

macro_rules! assert_err {
    ($expr: expr) => {
        assert!(($expr).is_err())
    };
    ($expr: expr, $err: ident) => {
        assert_eq!(($expr).err(), Some(Error::$err))
    };
}

fn make_disk() -> std::io::Result<RamDisk> {
    let path = std::env::current_dir()?.join(IMG_PATH);
    println!("Loading disk image from {:?} ...", path);
    let data = std::fs::read(path)?;
    println!("size = {} bytes", data.len());
    Ok(RamDisk::from(&data))
}

fn test_read_write_file() -> Result<()> {
    let fname = "///very/long//.././long//./path/./test.txt";
    println!("read and write file {:?}:", fname);

    // read and write
    let mut file = File::options().read(true).write(true).open(fname)?;
    let file_size = file.metadata()?.len();
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    print!("{}", contents);
    assert_eq!(contents.len(), file_size as usize);
    assert_eq!(file.write(b"Hello, world!\n")?, 14); // append
    drop(file);

    // read again and check
    let new_contents = fs::read_to_string(fname)?;
    print!("{}", new_contents);
    assert_eq!(new_contents, contents + "Hello, world!\n");

    // append and check
    let mut file = OpenOptions::new().append(true).open(fname)?;
    assert_eq!(file.write(b"new line\n")?, 9);
    drop(file);

    let new_contents2 = fs::read_to_string(fname)?;
    print!("{}", new_contents2);
    assert_eq!(new_contents2, new_contents + "new line\n");

    // open a non-exist file
    assert_err!(File::open("/not/exist/file"), NotFound);

    println!("test_read_write_file() OK!");
    Ok(())
}

fn test_read_dir() -> Result<()> {
    let dir = "/././//./";
    println!("list directory {:?}:", dir);
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        println!("   {}", entry.file_name());
    }
    println!("test_read_dir() OK!");
    Ok(())
}

fn test_file_permission() -> Result<()> {
    let fname = "./short.txt";
    println!("test permission {:?}:", fname);

    // write a file that open with read-only mode
    let mut buf = [0; 256];
    let mut file = File::open(fname)?;
    let n = file.read(&mut buf)?;
    assert_err!(file.write(&mut buf), PermissionDenied);
    drop(file);

    // read a file that open with write-only mode
    let mut file = File::create(fname)?;
    assert_err!(file.read(&mut buf), PermissionDenied);
    assert!(file.write(&buf[..n]).is_ok());
    drop(file);

    // open with empty options
    assert_err!(OpenOptions::new().open(fname), InvalidInput);

    // read as a directory
    assert_err!(fs::read_dir(fname), NotADirectory);
    assert_err!(fs::read("short.txt/"), NotADirectory);
    assert_err!(fs::metadata("/short.txt/"), NotADirectory);

    // create as a directory
    assert_err!(fs::write("error/", "should not create"), NotADirectory);
    assert_err!(fs::metadata("error/"), NotFound);
    assert_err!(fs::metadata("error"), NotFound);

    // read/write a directory
    assert_err!(fs::read_to_string("/dev"), IsADirectory);
    assert_err!(fs::write(".", "test"), IsADirectory);

    println!("test_file_permisson() OK!");
    Ok(())
}

fn test_devfs() -> Result<()> {
    const N: usize = 32;
    let mut buf = [1; N];

    // read and write /dev/null
    let mut file = File::options().read(true).write(true).open("/dev/./null")?;
    assert_eq!(file.read_to_end(&mut Vec::new())?, 0);
    assert_eq!(file.write(&buf)?, N);
    assert_eq!(buf, [1; N]);

    // read and write /dev/zero
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("////dev/zero")?;
    assert_eq!(file.read(&mut buf)?, N);
    assert!(file.write_all(&buf).is_ok());
    assert_eq!(buf, [0; N]);

    // list /dev
    let dirents = fs::read_dir("/dev")?
        .map(|e| e.unwrap().file_name())
        .collect::<Vec<_>>();
    assert!(dirents.contains(&"null".into()));
    assert!(dirents.contains(&"zero".into()));

    // stat /dev
    let dname = "/dev";
    let dir = File::open(dname)?;
    let md = dir.metadata()?;
    println!("metadata of {:?}: {:?}", dname, md);
    assert_eq!(md.file_type(), FileType::Dir);
    assert!(!md.is_file());
    assert!(md.is_dir());

    // stat /dev/foo/bar
    let fname = "/dev/.//./foo//./././bar";
    let file = File::open(fname)?;
    let md = file.metadata()?;
    println!("metadata of {:?}: {:?}", fname, md);
    assert_eq!(md.file_type(), FileType::CharDevice);
    assert!(!md.is_dir());

    println!("test_devfs() OK!");
    Ok(())
}

#[test]
fn test_axfs() {
    axtask::init_scheduler(); // call this to use `axsync::Mutex`.

    let disk = make_disk().expect("failed to load disk image");
    axfs::init_filesystems(disk);

    test_read_write_file().expect("test_read_write_file() failed");
    test_read_dir().expect("test_read_dir() failed");
    test_file_permission().expect("test_file_permission() failed");
    test_devfs().expect("test_devfs() failed");
}
