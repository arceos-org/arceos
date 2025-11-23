#[cfg(target_os = "hermit")]
use arceos_rust as _;

use std::io::{self, prelude::*};
use std::net::{TcpStream, ToSocketAddrs};

#[cfg(feature = "dns")]
const DEST: &str = "ident.me:80";
#[cfg(not(feature = "dns"))]
const DEST: &str = "65.108.151.63:80";

const REQUEST: &str = "\
GET / HTTP/1.1\r\n\
Host: ident.me\r\n\
Accept: */*\r\n\
\r\n";

fn client() -> io::Result<()> {
    #[cfg(feature = "dns")]
    println!("resolving host '{}':", DEST);

    for addr in DEST.to_socket_addrs()? {
        println!("dest: {} ({})", DEST, addr);
    }

    let mut stream = TcpStream::connect(DEST)?;
    stream.write_all(REQUEST.as_bytes())?;
    let mut buf = [0; 2048];
    let n = stream.read(&mut buf)?;
    let response = core::str::from_utf8(&buf[..n]).unwrap();
    println!("{}", response); // longer response need to handle tcp package problems.
    Ok(())
}

fn main() {
    println!("Hello, simple http client!");
    client().expect("test http client failed");
}
