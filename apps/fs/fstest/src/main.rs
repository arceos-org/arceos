#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;

use alloc::{string::String, vec::Vec};

use libax::fs::{self, File, FileType};
use libax::io::{self, prelude::*};

fn test_read_write_file() -> io::Result {
    let fname = "///very/long//.././long//./path/./test.txt";
    println!("read file {:?}:", fname);

    let mut file = File::open(fname)?;
    let file_size = file.metadata()?.len();
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    print!("{}", contents);
    assert_eq!(contents.len(), file_size as usize);

    file.write(b"Hello, world!")?;
    file.read_to_string(&mut contents)?;
    print!("{}", contents);

    println!("test_read_write_file() OK!");
    Ok(())
}

fn test_read_dir() -> io::Result {
    let dir = "/././//./";
    println!("list directory {:?}:", dir);
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        println!("   {}", entry.file_name());
    }
    println!("test_read_dir() OK!");
    Ok(())
}

fn test_devfs() -> io::Result {
    const N: usize = 32;
    let mut buf = [1; N];

    let mut file = File::open("/dev/./null")?;
    assert_eq!(file.read_to_end(&mut Vec::new())?, 0);
    assert_eq!(file.write(&buf)?, N);
    assert_eq!(buf, [1; N]);

    let mut file = File::open("/dev/zero")?;
    assert_eq!(file.read(&mut buf)?, N);
    assert_eq!(file.write_all(&buf), Ok(()));
    assert_eq!(buf, [0; N]);

    let dirents = fs::read_dir("/dev")?
        .map(|e| e.unwrap().file_name())
        .collect::<Vec<_>>();
    assert_eq!(dirents, ["foo", "null", "zero"]);

    let fname = "/dev/.//./foo//./././bar";
    let file = File::open(fname)?;
    let md = file.metadata()?;
    println!("metadata of {:?}: {:?}", fname, md);
    assert_eq!(md.file_type(), FileType::CharDevice);
    assert!(!md.is_dir());

    let dname = "/dev";
    let dir = File::open(dname)?;
    let md = dir.metadata()?;
    println!("metadata of {:?}: {:?}", dname, md);
    assert_eq!(md.file_type(), FileType::Dir);
    assert!(!md.is_file());
    assert!(md.is_dir());

    println!("test_devfs() OK!");
    Ok(())
}

#[no_mangle]
fn main() {
    println!("Running filesystem tests...");

    test_read_write_file().expect("test_read_write_file() failed");
    test_read_dir().expect("test_read_dir() failed");
    test_devfs().expect("test_devfs() failed");

    println!("Filesystem tests OK!");
}
