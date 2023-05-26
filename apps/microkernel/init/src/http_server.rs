//! Simple HTTP server.
//!
//! Benchmark with [Apache HTTP server benchmarking tool](https://httpd.apache.org/docs/2.4/programs/ab.html):
//!
//! ```
//! ab -n 5000 -c 20 http://X.X.X.X:5555/
//! ```

extern crate alloc;

use libax::{axerrno::AxResult, io::File, task::yield_now};

const LOCAL_IP: &str = "10.0.2.15";
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

fn http_server(stream: File) -> AxResult {
    let mut buf = [0u8; 1024];
    stream.read(&mut buf)?;

    let reponse = alloc::format!(header!(), CONTENT.len(), CONTENT);
    stream.write(reponse.as_bytes())?;

    Ok(())
}

fn accept_loop() -> AxResult {
    let (addr, port) = (LOCAL_IP, LOCAL_PORT);
    let path = &alloc::format!("tcp:/{}/{}", addr, port);
    println!("{}", path);
    let listener = File::create(path).unwrap();
    println!("listen on: {}", path);

    let mut i = 0;
    loop {
        match listener.dup("accept") {
            Ok(stream) => {
                info!("new client {}: ", i);
                libax::task::spawn(move || match http_server(stream) {
                    Err(e) => error!("client connection error: {:?}", e),
                    Ok(()) => info!("client {} closed successfully", i),
                });
            }
            Err(libax::axerrno::AxError::Again) => yield_now(),
            Err(e) => return Err(e),
        }
        i += 1;
    }
}

pub fn main() {
    println!("Hello, ArceOS HTTP server!");
    accept_loop().expect("test HTTP server failed");
}
