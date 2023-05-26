extern crate alloc;

use alloc::{vec::Vec, format};
use libax::io::{self, prelude::*, File};

const DEST_HOST: &str = "ident.me";
const DEST_IP: &str = "49.12.234.183";
const REQUEST: &str = "\
GET / HTTP/1.1\r\n\
Host: ident.me\r\n\
Accept: */*\r\n\
\r\n";

fn get_addr() -> (&'static str, u16) {
    (DEST_IP, 80)
}

fn client() -> io::Result {
    let mut stream = File::open(&format!("tcp:/{}/{}", get_addr().0, get_addr().1))?;
    stream.write(REQUEST.as_bytes())?;
    let mut buf = [0; 2048];
    let n = stream.read(&mut buf)?;
    let response = core::str::from_utf8(&buf[..n]).unwrap();
    println!("{}", response); // longer response need to handle tcp package problems.
    Ok(())
}

pub fn main() {
    println!("Hello, simple http client!");
    client().expect("test http client failed");
}
