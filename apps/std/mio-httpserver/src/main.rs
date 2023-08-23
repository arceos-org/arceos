//! Simple HTTP server base on [Mio](https://docs.rs/mio/0.8.8/mio/).
//!
//! Benchmark with [Apache HTTP server benchmarking tool](https://httpd.apache.org/docs/2.4/programs/ab.html):
//!
//! ```
//! ab -n 5000 -c 20 http://X.X.X.X:5555/
//! ```

use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Registry, Token};
use std::collections::HashMap;
use std::io::{self, Read, Write};

// Setup some tokens to allow us to identify which event is for which socket.
const SERVER: Token = Token(0);

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

fn main() -> io::Result<()> {
    // Create a poll instance.
    let mut poll = Poll::new()?;
    // Create storage for events.
    let mut events = Events::with_capacity(128);

    // Setup the TCP server socket.
    let addr = format!("{LOCAL_IP}:{LOCAL_PORT}").parse().unwrap();
    let mut server = TcpListener::bind(addr)?;

    // Register the server with poll we can receive events for it.
    poll.registry()
        .register(&mut server, SERVER, Interest::READABLE)?;

    // Map of `Token` -> `TcpStream`.
    let mut connections = HashMap::new();
    // Unique token for each incoming connection.
    let mut unique_token = Token(SERVER.0 + 1);

    println!("Hello, mio HTTP server!");
    println!("listen on: http://{}/", server.local_addr()?);

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER => loop {
                    // Received an event for the TCP server socket, which
                    // indicates we can accept an connection.
                    let (mut connection, address) = match server.accept() {
                        Ok((connection, address)) => (connection, address),
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            // If we get a `WouldBlock` error we know our
                            // listener has no more incoming connections queued,
                            // so we can return to polling and wait for some
                            // more.
                            break;
                        }
                        Err(e) => {
                            // If it was any other kind of error, something went
                            // wrong and we terminate with an error.
                            return Err(e);
                        }
                    };

                    let token = next(&mut unique_token);
                    info!("new client {}: {}", token.0, address);
                    poll.registry()
                        .register(&mut connection, token, Interest::READABLE)?;

                    connections.insert(token, connection);
                },
                token => {
                    // Maybe received an event for a TCP connection.
                    let done = if let Some(connection) = connections.get_mut(&token) {
                        handle_connection_event(poll.registry(), connection, event)?
                    } else {
                        // Sporadic events happen, we can safely ignore them.
                        false
                    };
                    if done {
                        info!("client {} closed successfully", token.0);
                        if let Some(mut connection) = connections.remove(&token) {
                            poll.registry().deregister(&mut connection)?;
                        }
                    }
                }
            }
        }
    }
}

fn next(current: &mut Token) -> Token {
    let next = current.0;
    current.0 += 1;
    Token(next)
}

/// Returns `true` if the connection is done.
fn handle_connection_event(
    registry: &Registry,
    connection: &mut TcpStream,
    event: &Event,
) -> io::Result<bool> {
    if event.is_readable() {
        let mut received_data = vec![0; 4096];
        match connection.read(&mut received_data) {
            Ok(0) => return Ok(true),
            Ok(_n) => registry.reregister(connection, event.token(), Interest::WRITABLE)?,
            Err(ref err) if would_block(err) => {}
            Err(err) => return Err(err),
        }
    }

    if event.is_writable() {
        let response = format!(header!(), CONTENT.len(), CONTENT);
        match connection.write(response.as_bytes()) {
            Ok(n) if n < response.len() => return Err(io::ErrorKind::WriteZero.into()),
            Ok(_) => return Ok(true),
            Err(ref err) if would_block(err) => {}
            Err(err) => return Err(err),
        }
    }

    Ok(false)
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}
