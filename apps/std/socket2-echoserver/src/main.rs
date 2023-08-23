//! Configuration needed to run this test (in the root Makefile):
//! A=apps/std/socket2-echoserver STD=y NET=y
use socket2::{Domain, Socket, Type};
use std::io::Write;
use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::str::from_utf8;

const DATA: &[u8] = b"hello world";
const BUF_SIZE: usize = 4096;
const ADDR: &str = "0.0.0.0:5555";

fn main() {
    env_logger::init();
    test()
}

fn test() {
    let mut recv_buffer = [MaybeUninit::<u8>::new(0); BUF_SIZE];
    let mut send_buffer = [0u8; BUF_SIZE];

    let socket = Socket::new(Domain::IPV4, Type::STREAM, None).unwrap();
    let addr: SocketAddr = ADDR.parse::<SocketAddr>().unwrap();
    println!("addr {:#?}", addr);
    #[cfg(not(target_os = "arceos"))]
    let addr: socket2::SockAddr = addr.into();
    socket.bind(&addr).unwrap();
    socket.listen(128).unwrap();

    println!("---------- socket2 echoserver ----------");
    println!("type `nc {}` at another terminal:", ADDR);

    loop {
        let (mut connection, sockaddr) = loop {
            if let Ok(result) = socket.accept() {
                break result;
            } else {
                println!("user got a Err from accept, try again");
            }
        };
        #[cfg(target_os = "arceos")]
        println!("Accepted connection from: {}", sockaddr);
        #[cfg(not(target_os = "arceos"))]
        println!(
            "Accepted connection from: {}",
            sockaddr.as_socket().unwrap()
        );
        connection.write_all(DATA).unwrap();

        loop {
            let n = connection.recv(&mut recv_buffer).unwrap();
            if n == 0 {
                break;
            }
            for i in 0..n {
                send_buffer[i] = unsafe { recv_buffer[i].assume_init() };
            }
            let received_data = &send_buffer[..n];
            if let Ok(str_buf) = from_utf8(received_data) {
                println!("Received data({}B): {}", n, str_buf.trim_end());
                connection.write_all(received_data).unwrap();
            } else {
                println!("Received (none UTF-8) data: {:?}", received_data);
            }
        }
    }
}
