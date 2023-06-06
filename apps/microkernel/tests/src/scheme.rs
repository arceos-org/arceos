#![allow(unused)]

use core::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use libax::task::{self, exit, spawn, yield_now};
extern crate alloc;

static FINISHED: AtomicUsize = AtomicUsize::new(0);

pub mod server {
    use core::{cell::RefCell, sync::atomic::Ordering};

    use alloc::{collections::BTreeMap, vec::Vec};
    use libax::{
        axerrno::{ax_err, AxError, AxResult},
        io::File,
        scheme::{Packet, Scheme},
    };

    use crate::scheme::FINISHED;

    struct ServerInner {
        data: Vec<u8>,
        fds: BTreeMap<usize, usize>, // fd, offset
        next_fd: usize,
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
        inner: RefCell<ServerInner>,
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
                return ax_err!(WouldBlock);
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
            inner
                .fds
                .remove(&id)
                .ok_or(AxError::BadFileDescriptor)
                .map(|_| 0)
        }
    }
    impl Server {
        fn new() -> Self {
            Self {
                inner: ServerInner::new().into(),
            }
        }

        fn finished(&self) -> bool {
            let inner = self.inner.borrow();
            inner.next_fd == 3 && inner.fds.is_empty()
        }
    }

    pub fn main() {
        println!("Server started!");
        let server = Server::new();
        let mut channel = File::create(":/test").unwrap();

        loop {
            let mut packet: Packet = Packet::default();
            assert_eq!(
                channel.read_data(&mut packet).unwrap(),
                core::mem::size_of::<Packet>()
            );
            server.handle(&mut packet);
            assert_eq!(
                channel.write_data(&packet).unwrap(),
                core::mem::size_of::<Packet>()
            );

            if server.finished() {
                break;
            }
        }
        FINISHED.fetch_add(1, Ordering::Relaxed);
    }
}

mod client {
    use core::{sync::atomic::Ordering, time::Duration};

    use alloc::vec;
    use libax::{
        axerrno::AxError,
        io::{File, Read, Write},
        task::{self, spawn},
    };

    pub fn main() {
        spawn(|| {
            println!("Client sender started!");
            let mut file = File::create("test:/test").unwrap();
            let data = vec![
                "Hello", " ", "world", "\n", "Goodbye", " ", "world", "\n", "\0",
            ];
            for item in &data {
                println!("Send: '{}'", item);
                assert!(file.write(item.as_bytes()).unwrap() == item.len());
            }
            super::FINISHED.fetch_add(1, Ordering::Relaxed);
        });
        spawn(|| {
            println!("Client receiver started!");
            let mut file = File::create("test:/test").unwrap();
            let mut buffer = [0u8; 8];
            'a: loop {
                let len = match file.read(&mut buffer) {
                    Ok(len) => len,
                    Err(AxError::WouldBlock) => {
                        task::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(e) => {
                        panic!("{:?}", e);
                    }
                };
                print!("Received: ");
                for i in &buffer[..len] {
                    if *i == b'\0' {
                        break 'a;
                    }
                    print!("{}({}), ", i, char::from_u32(*i as u32).unwrap());
                }
                println!("");
            }
            println!("ENDED");
            super::FINISHED.fetch_add(1, Ordering::Relaxed);
        });
    }
}
pub fn main() {
    spawn(server::main);
    task::sleep(Duration::from_millis(100));
    spawn(client::main);
    while FINISHED.load(Ordering::Relaxed) != 3 {
        yield_now();
    }
    exit(0);
}
