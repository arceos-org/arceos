#![no_std]
#![no_main]

use libax::{
    fs::api::{File, OpenOptions},
    io::{Seek, Write},
};

#[no_mangle]
fn main() -> i32 {
    let mut file: OpenOptions = File::options();
    file.read(true);
    file.write(true);
    file.create(true);
    let mut new_file = file.open("/XXX").unwrap();
    let buf = "hello";
    new_file.write(buf.as_bytes());
    new_file.write(buf.as_bytes());
    let pos = new_file.seek(libax::io::SeekFrom::Current(0)).unwrap();
    let len = new_file.seek(libax::io::SeekFrom::End(0)).unwrap();
    libax::println!("len: {}", len);
    new_file.seek(libax::io::SeekFrom::Start(pos)).unwrap();
    let mut file: OpenOptions = File::options();
    file.read(true);
    new_file.write(buf.as_bytes());
    new_file.write(buf.as_bytes());

    let mut new_file2 = file.open("/XXX").unwrap();
    drop(new_file);
    let pos = new_file2.seek(libax::io::SeekFrom::Current(0)).unwrap();
    let len = new_file2.seek(libax::io::SeekFrom::End(0)).unwrap();
    libax::println!("len2: {}", len);
    new_file2.seek(libax::io::SeekFrom::Start(pos)).unwrap();
    0
}
