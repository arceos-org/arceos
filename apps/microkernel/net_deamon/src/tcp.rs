use core::{
    str::FromStr,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::collections::BTreeMap;
use axnet::{IpAddr, SocketAddr, TcpSocket};
use libax::{
    axerrno::{ax_err, AxError, AxResult},
    io::File,
    scheme::{Packet, Scheme},
    Mutex, OpenFlags,
};

extern crate alloc;

struct TcpScheme {
    handles: Mutex<BTreeMap<usize, TcpSocket>>,
    next_id: AtomicUsize,
}
impl TcpScheme {
    fn new() -> TcpScheme {
        TcpScheme {
            handles: Mutex::new(BTreeMap::new()),
            next_id: 1.into(),
        }
    }
}
impl Scheme for TcpScheme {
    fn open(&self, path: &str, flags: usize, _uid: u32, _gid: u32) -> AxResult<usize> {
        info!("OPEN {}", path);
        let mut path = path.trim_matches('/').split('/');
        let addr = match (path.next(), path.next()) {
            (Some(addr), Some(port)) => SocketAddr::new(
                IpAddr::from_str(&addr).map_err(|_| AxError::NotFound)?,
                port.parse().map_err(|_| AxError::NotFound)?,
            ),
            _ => return ax_err!(NotFound),
        };

        let mut socket = TcpSocket::new();
        let flags = OpenFlags::from_bits_truncate(flags);

        if flags.contains(OpenFlags::CREATE) {
            // listen
            socket.bind(addr)?;
            socket.listen()?;
        } else {
            // connect
            socket.connect(addr)?;
        }
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.handles.lock().insert(id, socket);
        Ok(id)
    }

    fn close(&self, id: usize) -> AxResult<usize> {
        info!("CLOSE {}", id);
        self.handles
            .lock()
            .remove(&id)
            .ok_or(AxError::BadFileDescriptor)?
            .shutdown()?;
        Ok(0)
    }

    fn read(&self, id: usize, buf: &mut [u8]) -> AxResult<usize> {
        info!("READ {}", id);
        self.handles
            .lock()
            .get(&id)
            .ok_or(AxError::BadFileDescriptor)?
            .recv(buf)
    }

    fn write(&self, id: usize, buf: &[u8]) -> AxResult<usize> {
        info!("WRITE {}", id);
        self.handles
            .lock()
            .get(&id)
            .ok_or(AxError::BadFileDescriptor)?
            .send(buf)
    }
    fn dup(&self, old_id: usize, buf: &[u8]) -> AxResult<usize> {
        if buf != b"accept" {
            return ax_err!(InvalidInput);
        }
        info!("DUP {}", old_id);
        let mut handles = self.handles.lock();
        let handle = handles.get_mut(&old_id).ok_or(AxError::BadFileDescriptor)?;

        let new_handle = handle.accept()?;
        println!("{:?}", new_handle.peer_addr());
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        handles.insert(id, new_handle);
        Ok(id)
    }
}
pub fn start_tcp() {
    let tcp = TcpScheme::new();
    let channel = File::create(":/tcp").unwrap();
    libax::println!("TCP deamon started!");
    loop {
        let mut packet: Packet = Packet::default();
        assert_eq!(
            channel.read_data(&mut packet).unwrap(),
            core::mem::size_of::<Packet>()
        );
        tcp.handle(&mut packet);
        assert_eq!(
            channel.write_data(&packet).unwrap(),
            core::mem::size_of::<Packet>()
        );
    }
}
