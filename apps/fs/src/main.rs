#![no_std]
#![no_main]

use libax::fs::{init_vfs, list, lseek, mkdir, open, read, write, FileMode, OpenFlags, SeekFrom};
use libax::println;

#[no_mangle]
fn main() {
    println!("VFS TEST.....");
    init_vfs();
    let _ = open(
        "/",
        OpenFlags::O_RDWR | OpenFlags::O_CREAT,
        FileMode::FMODE_WRITE,
    );
    let f1 = open(
        "/f1",
        OpenFlags::O_RDWR | OpenFlags::O_CREAT,
        FileMode::FMODE_WRITE,
    );
    assert!(f1.is_some());
    let f1 = f1.unwrap();
    let len = write(f1.clone(), "hello world".as_bytes()).unwrap();
    println!("write len:{}", len);

    let mut buf = [0u8; 20];
    let len = read(f1.clone(), &mut buf).unwrap();
    println!("read len:{}", len); // len=0 because the file's f_pos is at the end of file

    let pos = lseek(f1.clone(), SeekFrom::Start(0)).unwrap();
    assert_eq!(pos, 0);

    let len = read(f1.clone(), &mut buf).unwrap();
    println!("read len:{}", len); // len=11
    println!("buf:{}", core::str::from_utf8(&buf).unwrap());

    mkdir("./dir1", FileMode::FMODE_WRITE).unwrap();

    println!("list root dir:");
    list("/").unwrap().iter().for_each(|x| println!("{}", x));
}
