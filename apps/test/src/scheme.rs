#![allow(unused)]

use core::time::Duration;

use libax::task::{spawn_fn, self};
extern crate alloc;

pub mod server {
    use core::cell::RefCell;

    use alloc::{vec::Vec, collections::BTreeMap};
    use libax::{scheme::{Scheme, Packet}, axerrno::{AxResult, AxError, ax_err}, io::File};

    struct ServerInner {
        data: Vec<u8>,
        fds: BTreeMap<usize, usize>, // fd, offset
        next_fd: usize
    }
    impl ServerInner {
        fn new() -> Self {
            Self {
                data: Vec::new(),
                fds: BTreeMap::new(),
                next_fd: 1,
            }
        }
    }
    struct Server {
        inner: RefCell<ServerInner>
    }
    impl Scheme for Server {
        fn open(&self, _path: &str, flags: usize, uid: u32, gid: u32) -> AxResult<usize> {
            // To simplify, all client share one file
            let mut inner = self.inner.borrow_mut();
            let id = inner.next_fd;
            inner.next_fd += 1;
            inner.fds.insert(id, 0);
            Ok(id)
        }
        fn read(&self, id: usize, buf: &mut [u8]) -> AxResult<usize> {
            let mut inner = self.inner.borrow_mut();
            let offset = *inner.fds.get(&id).ok_or(AxError::BadFileDescriptor)?;
            let end = (offset + buf.len()).min(inner.data.len());
            if offset == end {
                // We do not have coroutines, so that we must return EAGAIN
                // to let user wait manually
                return ax_err!(Again);
            }
            let len = end - offset;
            buf[..len].copy_from_slice(&inner.data[offset..end]);
            inner.fds.insert(id, end);
            Ok(len)
        }
        fn write(&self, id: usize, buf: &[u8]) -> AxResult<usize> {
            let mut inner = self.inner.borrow_mut();
            inner.data.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn close(&self, id: usize) -> AxResult<usize> {
            let mut inner = self.inner.borrow_mut();
            inner.fds.remove(&id).ok_or(AxError::BadFileDescriptor).map(|_| 0)
        }
    }
    impl Server {
        fn new() -> Self {
            Self {
                inner: ServerInner::new().into(),
            }
        }
    }

    pub fn main() {
        println!("Server started!");
        let server = Server::new();
        let channel = File::create(":/test").unwrap();
        
        loop {
            let mut packet: Packet = Packet::default();
            assert_eq!(channel.read_data(&mut packet).unwrap(), core::mem::size_of::<Packet>());
            server.handle(&mut packet);
            assert_eq!(channel.write_data(&packet).unwrap(), core::mem::size_of::<Packet>());
        }
    }
}

mod client {
    use core::time::Duration;

    use alloc::vec;
    use libax::{task::{spawn, self}, io::File, axerrno::AxError};

    pub fn main() {
        spawn(|| {
            println!("Client sender started!");
            let file = File::create("test:/test").unwrap();
            let data = vec!["Hello", " ", "world", "\n", "Goodbye", " ", "world", "\n"];
            for item in &data {
                println!("Send: '{}'", item);
                assert!(file.write(item.as_bytes()).unwrap() == item.len());
            }
        });
        spawn(|| {
            println!("Client receiver started!");
            let file = File::create("test:/test").unwrap();
            let mut buffer = [0u8; 8];
            loop {
                let len = match file.read(&mut buffer) {
                    Ok(len) => len,
                    Err(AxError::Again) => {
                        task::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(e) => {
                        panic!("{:?}", e);
                    }
                };
                print!("Received: ");
                for i in &buffer[..len] {
                    print!("{}({}), ", i, char::from_u32(*i as u32).unwrap());
                }
                println!("");
                
            }
        });
        loop {
            task::sleep(Duration::from_secs(1));
        }
    }
}
pub fn main() {
    spawn_fn(server::main);
    task::sleep(Duration::from_millis(100));
    spawn_fn(client::main);
    loop {
        task::sleep(Duration::from_secs(1));
    }
}
