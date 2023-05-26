extern crate alloc;
use core::{
    str::FromStr,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::collections::BTreeMap;
use axnet::{IpAddr, SocketAddr, UdpSocket};
use libax::{
    axerrno::{AxError, AxResult, ax_err},
    scheme::{Scheme, Packet},
    Mutex, OpenFlags, io::File,
};

struct UdpScheme {
    handles: Mutex<BTreeMap<usize, UdpSocket>>,
    next_id: AtomicUsize,
}
impl UdpScheme {
    fn new() -> UdpScheme {
        UdpScheme {
            handles: Mutex::new(BTreeMap::new()),
            next_id: 1.into(),
        }
    }
}
impl Scheme for UdpScheme {
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

        let mut socket = UdpSocket::new();
        let flags = OpenFlags::from_bits_truncate(flags);

        if flags.contains(OpenFlags::CREATE) {
            // listen
            socket.bind(addr)?;
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
        self.handles.lock()
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
}
pub fn start_udp() {
    let udp = UdpScheme::new();
    let channel = File::create(":/udp").unwrap();
    libax::println!("UDP deamon started!");
    loop {
        let mut packet: Packet = Packet::default();
        assert_eq!(channel.read_data(&mut packet).unwrap(), core::mem::size_of::<Packet>());
        udp.handle(&mut packet);
        assert_eq!(channel.write_data(&packet).unwrap(), core::mem::size_of::<Packet>());
    }
}

