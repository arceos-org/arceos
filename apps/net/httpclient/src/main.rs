#![no_std]
#![no_main]

#[macro_use]
extern crate axruntime;

use core::str::FromStr;

use axerror::AxResult;
use axnet::io::{Read, Write};
use axnet::{IpAddr, TcpStream};

const DEST_IP: &str = "49.12.234.183"; // ident.me
const REQUEST: &str = "\
GET / HTTP/1.1\r\n\
Host: ident.me\r\n\
Accept: */*\r\n\
\r\n";

fn client() -> AxResult {
    let (addr, port) = (IpAddr::from_str(DEST_IP).unwrap(), 80);
    let mut stream = TcpStream::connect((addr, port).into())?;
    stream.write(REQUEST.as_bytes())?;

    let mut buf = [0; 1024];
    let n = stream.read(&mut buf)?;
    let response = core::str::from_utf8(&buf[..n]).unwrap();
    println!("{}", response);

    Ok(())
}

#[no_mangle]
fn main() {
    println!("Hello, simple http client!");
    client().expect("test http client failed");
}
