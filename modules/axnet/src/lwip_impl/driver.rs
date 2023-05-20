use super::LWIP_MUTEX;
use crate::IpAddr;
use alloc::collections::VecDeque;
use axdriver::prelude::*;
use axsync::Mutex;
use core::{cell::RefCell, ffi::c_void};
use driver_net::{DevError, NetBufferBox, NetBufferPool};
use lazy_init::LazyInit;
use lwip_rust::bindings::{
    err_enum_t_ERR_OK, err_enum_t_ERR_WOULDBLOCK, err_t, etharp_output, ethernet_input,
    ethip6_output, ip4_addr_t, lwip_htonl, lwip_init, netif, netif_add,
    netif_create_ip6_linklocal_address, netif_set_default, netif_set_link_up, netif_set_up, pbuf,
    pbuf_alloc, pbuf_layer_PBUF_RAW, pbuf_type_PBUF_POOL, sys_check_timeouts, NETIF_FLAG_BROADCAST,
    NETIF_FLAG_ETHARP, NETIF_FLAG_ETHERNET,
};

const RX_BUF_QUEUE_SIZE: usize = 64;

const NET_BUF_LEN: usize = 1526;
const NET_BUF_POOL_SIZE: usize = 128;

static NET_BUF_POOL: LazyInit<NetBufferPool> = LazyInit::new();

struct NetifWrapper(netif);
unsafe impl Send for NetifWrapper {}

struct DeviceWrapper {
    inner: RefCell<AxNetDevice>, // use `RefCell` is enough since it's wrapped in `Mutex` in `InterfaceWrapper`.
    rx_buf_queue: VecDeque<NetBufferBox<'static>>,
}

impl DeviceWrapper {
    fn new(inner: AxNetDevice) -> Self {
        Self {
            inner: RefCell::new(inner),
            rx_buf_queue: VecDeque::with_capacity(RX_BUF_QUEUE_SIZE),
        }
    }

    fn poll(&mut self) {
        while self.rx_buf_queue.len() < RX_BUF_QUEUE_SIZE {
            match self.inner.borrow_mut().receive() {
                Ok(buf) => {
                    self.rx_buf_queue.push_back(buf);
                }
                Err(DevError::Again) => break, // TODO: better method to avoid error type conversion
                Err(err) => {
                    warn!("receive failed: {:?}", err);
                    break;
                }
            }
        }
    }

    fn receive(&mut self) -> Option<NetBufferBox<'static>> {
        self.rx_buf_queue.pop_front()
    }
}

struct InterfaceWrapper {
    dev: Mutex<DeviceWrapper>,
    netif: Mutex<NetifWrapper>,
}

impl InterfaceWrapper {
    fn poll(&self) {
        self.dev.lock().poll();
        loop {
            let buf_receive = self.dev.lock().receive();
            if let Some(buf) = buf_receive {
                trace!("RECV {} bytes: {:02X?}", buf.packet().len(), buf.packet());

                // Copy buf to pbuf
                let len = buf.packet().len();
                let p = unsafe { pbuf_alloc(pbuf_layer_PBUF_RAW, len as u16, pbuf_type_PBUF_POOL) };
                if p.is_null() {
                    warn!("pbuf_alloc failed");
                    continue;
                }
                let payload = unsafe { (*p).payload };
                let payload = unsafe { core::slice::from_raw_parts_mut(payload as *mut u8, len) };
                payload.copy_from_slice(buf.packet());
                let res = self.dev.lock().inner.borrow_mut().recycle_rx_buffer(buf);
                match res {
                    Ok(_) => (),
                    Err(err) => {
                        warn!("recycle_rx_buffer failed: {:?}", err);
                    }
                }

                debug!("ethernet_input");
                let mut netif = self.netif.lock();
                unsafe {
                    netif.0.input.unwrap()(p, &mut netif.0);
                }
            } else {
                break;
            }
        }
    }
}

extern "C" fn ethif_init(netif: *mut netif) -> err_t {
    debug!("ethif_init");
    unsafe {
        (*netif).name[0] = 'e' as i8;
        (*netif).name[1] = 'n' as i8;
        (*netif).num = 0;

        (*netif).output = Some(etharp_output);
        (*netif).output_ip6 = Some(ethip6_output);
        (*netif).linkoutput = Some(ethif_output);

        (*netif).mtu = 1500;
        (*netif).flags = 0;
        (*netif).flags = (NETIF_FLAG_BROADCAST | NETIF_FLAG_ETHARP | NETIF_FLAG_ETHERNET) as u8;
    }
    err_enum_t_ERR_OK as err_t
}

extern "C" fn ethif_output(netif: *mut netif, p: *mut pbuf) -> err_t {
    debug!("ethif_output");
    let ethif = unsafe { &mut *((*netif).state as *mut _ as *mut InterfaceWrapper) };
    let dev_wrapper = ethif.dev.lock();
    let mut dev = dev_wrapper.inner.borrow_mut();

    if dev.can_transmit() {
        let tot_len = unsafe { (*p).tot_len };
        let mut tx_buf = NET_BUF_POOL.alloc().unwrap();
        dev.prepare_tx_buffer(&mut tx_buf, tot_len.into()).unwrap();

        // Copy pbuf chain to tx_buf
        let mut offset = 0;
        let mut q = p;
        while !q.is_null() {
            let len = unsafe { (*q).len } as usize;
            let payload = unsafe { (*q).payload };
            let payload = unsafe { core::slice::from_raw_parts(payload as *const u8, len) };
            tx_buf.packet_mut()[offset..offset + len].copy_from_slice(payload);
            offset += len;
            q = unsafe { (*q).next };
        }

        trace!(
            "SEND {} bytes: {:02X?}",
            tx_buf.packet().len(),
            tx_buf.packet()
        );
        dev.transmit(&tx_buf).unwrap();
        err_enum_t_ERR_OK as err_t
    } else {
        err_enum_t_ERR_WOULDBLOCK as err_t
    }
}

static mut ETH0: LazyInit<InterfaceWrapper> = LazyInit::new();

fn ip4_addr_gen(a: u8, b: u8, c: u8, d: u8) -> ip4_addr_t {
    ip4_addr_t {
        addr: unsafe {
            lwip_htonl(((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | (d as u32))
                as u32
        },
    }
}

pub fn init(mut net_dev: AxNetDevice) {
    let pool = NetBufferPool::new(NET_BUF_POOL_SIZE, NET_BUF_LEN).unwrap();
    NET_BUF_POOL.init_by(pool);
    net_dev.fill_rx_buffers(&NET_BUF_POOL).unwrap();

    LWIP_MUTEX.init_by(Mutex::new(()));
    let _guard = LWIP_MUTEX.lock();

    let mut ipaddr: ip4_addr_t = ip4_addr_gen(10, 0, 2, 15); // QEMU user networking default IP
    let mut netmask: ip4_addr_t = ip4_addr_gen(255, 255, 255, 0);
    let mut gw: ip4_addr_t = ip4_addr_gen(10, 0, 2, 2); // QEMU user networking gateway

    let dev = net_dev;
    let mut netif: netif = unsafe { core::mem::zeroed() };
    netif.hwaddr_len = 6;
    netif.hwaddr = dev.mac_address().0;

    unsafe {
        ETH0.init_by(InterfaceWrapper {
            dev: Mutex::new(DeviceWrapper::new(dev)),
            netif: Mutex::new(NetifWrapper(netif)),
        });
    }

    unsafe {
        lwip_init();
        netif_add(
            &mut ETH0.netif.lock().0,
            &mut ipaddr,
            &mut netmask,
            &mut gw,
            &mut ETH0 as *mut _ as *mut c_void,
            Some(ethif_init),
            Some(ethernet_input),
        );
        netif_create_ip6_linklocal_address(&mut ETH0.netif.lock().0, 1);
        netif_set_link_up(&mut ETH0.netif.lock().0);
        netif_set_up(&mut ETH0.netif.lock().0);
        netif_set_default(&mut ETH0.netif.lock().0);
    }

    info!(
        "ETH0 IPv4 address: {}",
        IpAddr::from(unsafe { ETH0.netif.lock().0.ip_addr })
    );
    info!(
        "ETH0 IPv6 address: {}",
        IpAddr::from(unsafe { ETH0.netif.lock().0.ip6_addr[0] })
    );

    // let ipaddr: ip_addr_t = ip_addr_t {
    //     u_addr: ip_addr__bindgen_ty_1 { ip4: ipaddr },
    //     type_: lwip_ip_addr_type_IPADDR_TYPE_V4 as u8,
    // };
    // unsafe {
    //     lwiperf_start_tcp_server(&ipaddr, 5555, None, core::ptr::null_mut());
    // }
    drop(_guard);

    // loop {
    //     lwip_loop_once();
    // }
}

pub fn lwip_loop_once() {
    trace!("lwip_loop_once");
    let guard = LWIP_MUTEX.lock();
    unsafe {
        ETH0.poll();
        sys_check_timeouts();
    }
    drop(guard);
}
