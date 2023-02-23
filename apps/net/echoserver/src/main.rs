#![no_std]
#![no_main]

#[macro_use]
extern crate axruntime;
extern crate alloc;

use alloc::vec::Vec;
use core::str::FromStr;

use axerror::AxResult;
use axnet::io::{Read, Write};
use axnet::{IpAddr, TcpListener, TcpStream};

const LOCAL_IP: &str = "10.0.2.15";
const LOCAL_PORT: u16 = 5555;

fn reverse(buf: &[u8]) -> Vec<u8> {
    let mut lines = buf
        .split(|&b| b == b'\n')
        .map(Vec::from)
        .collect::<Vec<_>>();
    for line in lines.iter_mut() {
        line.reverse();
    }
    lines.join(&b'\n')
}

fn echo_server(mut stream: TcpStream) -> AxResult {
    let mut buf = [0u8; 1024];
    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            return Ok(());
        }
        stream.write_all(reverse(&buf[..n]).as_slice())?;
    }
}

fn accept_loop() -> AxResult {
    let (addr, port) = (IpAddr::from_str(LOCAL_IP).unwrap(), LOCAL_PORT);
    let mut listener = TcpListener::bind((addr, port).into())?;
    println!("listen on: {}", listener.local_addr().unwrap());

    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                println!("new client: {}", addr);
                axtask::spawn(|| match echo_server(stream) {
                    Err(e) => println!("client connection error: {:?}", e),
                    Ok(()) => println!("client closed successfully"),
                });
            }
            Err(e) => return Err(e),
        }
    }
}

#[no_mangle]
fn main() {
    println!("Hello, echo server!");
    accept_loop().expect("test echo server failed");
}
