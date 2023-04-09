# INTRODUCTION

| App | Extra modules | Enabled features | Description |
| [echoserver](apps/net/echoserver/) | axalloc, axdriver, axnet, axtask | alloc, paging, net, multitask | A multi-threaded TCP server that reverses messages sent by the client |

# RUN

**You might need to change the ip address (`LOCAL_TP`) in `apps/net/echoserver/src/main.rs`.**

The following example changes it to `127.0.0.1`.

```
make A=apps/net/echoserver NET=y run
...
Hello, echo server!
listen on: 127.0.0.1:5555
```

In another shell, use `telnet` to view the reversed echo message:

```
> telnet localhost 5555

Trying ::1...
telnet: connect to address ::1: Connection refused
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
hello
olleh
12345
54321
```

# STEPS

## step1

[init](./init.md)

After executed all initial actions, then arceos calls `main` function in `echoserver` app.

## step2

`main` calls `accept_loop()`, which will keep processing incoming tcp connection.

```rust
let (addr, port) = (IpAddr::from_str(LOCAL_IP).unwrap(), LOCAL_PORT);
let mut listener = TcpListener::bind((addr, port).into())?;
println!("listen on: {}", listener.local_addr().unwrap());

let mut i = 0;
loop {
    match listener.accept() {
        ...
        }
        Err(e) => return Err(e),
    }
    i += 1;
}
```

## step3

Once it receives a tcp connection. It will get a `(stream, addr)` pair from `libax::net`.
`main` task will spawn a task to reverse every package it receives.

```rust
Ok((stream, addr)) => {
    info!("new client {}: {}", i, addr);
    task::spawn(move || match echo_server(stream) {
        Err(e) => error!("client connection error: {:?}", e),
        Ok(()) => info!("client {} closed successfully", i),
    });
}
```

## step4

Reverse bytes in package it receives.

```rust
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

fn echo_server(mut stream: TcpStream) -> io::Result {
    let mut buf = [0u8; 1024];
    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            return Ok(());
        }
        stream.write_all(reverse(&buf[..n]).as_slice())?;
    }
}
```
