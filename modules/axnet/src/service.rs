use axhal::time::{NANOS_PER_MICROS, wall_time_nanos};
use smoltcp::{
    iface::{Interface, SocketSet},
    time::Instant,
    wire::{HardwareAddress, IpAddress},
};

use crate::router::Router;

fn now() -> Instant {
    Instant::from_micros_const((wall_time_nanos() / NANOS_PER_MICROS) as i64)
}

pub struct Service {
    pub iface: Interface,
    router: Router,
}
impl Service {
    pub fn new(mut router: Router) -> Self {
        let config = smoltcp::iface::Config::new(HardwareAddress::Ip);
        let iface = Interface::new(config, &mut router, now());

        Self { iface, router }
    }

    pub fn poll(&mut self, sockets: &mut SocketSet) {
        let timestamp = now();

        self.router.poll(timestamp);
        self.iface.poll(timestamp, &mut self.router, sockets);
        self.router.dispatch(timestamp);
    }

    pub fn get_source_address(&self, dst_addr: &IpAddress) -> IpAddress {
        let Some(rule) = self.router.table.lookup(dst_addr) else {
            panic!("no route to destination: {dst_addr}");
        };
        rule.src
    }

    // TODO(mivik): replace this method
    pub fn is_external(&self, dst_addr: &IpAddress) -> bool {
        self.router
            .table
            .lookup(dst_addr)
            .is_none_or(|it| it.dev > 0)
    }
}
