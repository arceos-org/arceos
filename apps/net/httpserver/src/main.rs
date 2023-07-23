//! Simple HTTP server.
//!
//! Benchmark with [Apache HTTP server benchmarking tool](https://httpd.apache.org/docs/2.4/programs/ab.html):
//!
//! ```
//! ab -n 5000 -c 20 http://X.X.X.X:5555/
//! ```

#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use std::io::{self, prelude::*};
use std::net::{TcpListener, TcpStream};
use std::thread;

const LOCAL_IP: &str = "0.0.0.0";
const LOCAL_PORT: u16 = 5555;

macro_rules! header {
    () => {
        "\
HTTP/1.1 200 OK\r\n\
Content-Type: text/html\r\n\
Content-Length: {}\r\n\
Connection: close\r\n\
\r\n\
{}"
    };
}

const CONTENT: &str = r#"<html>
<head>
  <title>Hello, ArceOS</title>
</head>
<body>
  <center>
    <h1>Hello, <a href="https://github.com/rcore-os/arceos">ArceOS</a></h1>
  </center>
  <hr>
  <center>
    <i>Powered by <a href="https://github.com/rcore-os/arceos/tree/main/apps/net/httpserver">ArceOS example HTTP server</a> v0.1.0</i>
  </center>
</body>
</html>
"#;

macro_rules! info {
    ($($arg:tt)*) => {
        match option_env!("LOG") {
            Some("info") | Some("debug") | Some("trace") => {
                print!("[INFO] {}\n", format_args!($($arg)*));
            }
            _ => {}
        }
    };
}

fn http_server(mut stream: TcpStream) -> io::Result<()> {
    let mut buf = [0u8; 4096];
    let _len = stream.read(&mut buf)?;

    let response = format!(header!(), CONTENT.len(), CONTENT);
    stream.write_all(response.as_bytes())?;

    Ok(())
}

fn accept_loop() -> io::Result<()> {
    let listener = TcpListener::bind((LOCAL_IP, LOCAL_PORT))?;
    println!("listen on: http://{}/", listener.local_addr().unwrap());

    let mut i = 0;
    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                info!("new client {}: {}", i, addr);
                thread::spawn(move || match http_server(stream) {
                    Err(e) => info!("client connection error: {:?}", e),
                    Ok(()) => info!("client {} closed successfully", i),
                });
            }
            Err(e) => return Err(e),
        }
        i += 1;
    }
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("Hello, ArceOS HTTP server!");
    accept_loop().expect("test HTTP server failed");
}
