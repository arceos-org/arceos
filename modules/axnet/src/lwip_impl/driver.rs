use super::LWIP_MUTEX;
use crate::{
    net_impl::addr::{mask_to_prefix, MacAddr},
    IpAddr,
};
use alloc::{boxed::Box, collections::VecDeque, sync::Arc};
use axdriver::prelude::*;
#[cfg(feature = "irq")]
use axdriver::register_interrupt_handler;
use axsync::Mutex;
use core::{cell::RefCell, ffi::c_void};
use driver_net::{DevError, NetBufferBox, NetBufferPool};
use lazy_init::LazyInit;
use lwip_rust::bindings::{
    err_enum_t_ERR_MEM, err_enum_t_ERR_OK, err_t, etharp_output, ethernet_input, ethip6_output,
    ip4_addr_t, lwip_htonl, lwip_init, netif, netif_add, netif_create_ip6_linklocal_address,
    netif_set_default, netif_set_link_up, netif_set_up, pbuf, pbuf_alloced_custom, pbuf_custom,
    pbuf_free, pbuf_layer_PBUF_RAW, pbuf_type_PBUF_REF, sys_check_timeouts, NETIF_FLAG_BROADCAST,
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

    #[cfg(feature = "irq")]
    fn ack_interrupt(&mut self) -> bool {
        unsafe { self.inner.as_ptr().as_mut().unwrap().ack_interrupt() }
    }
}

struct InterfaceWrapper {
    name: &'static str,
    dev: Arc<Mutex<DeviceWrapper>>,
    netif: Mutex<NetifWrapper>,
}

impl InterfaceWrapper {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn poll(&self) {
        self.dev.lock().poll();
        loop {
            let buf_receive = self.dev.lock().receive();
            if let Some(buf) = buf_receive {
                trace!("RECV {} bytes: {:02X?}", buf.packet().len(), buf.packet());

                let custom_pbuf = Box::new(CustomPbuf {
                    p: pbuf_custom {
                        pbuf: pbuf {
                            next: core::ptr::null_mut(),
                            payload: core::ptr::null_mut(),
                            tot_len: 0,
                            len: 0,
                            type_internal: 0,
                            flags: 0,
                            ref_: 0,
                            if_idx: 0,
                        },
                        custom_free_function: Some(pbuf_free_custom),
                    },
                    buf: Some(buf),
                    dev: self.dev.clone(),
                });
                let p = unsafe {
                    pbuf_alloced_custom(
                        pbuf_layer_PBUF_RAW,
                        custom_pbuf.buf.as_ref().unwrap().packet().len() as u16,
                        pbuf_type_PBUF_REF,
                        &custom_pbuf.p as *const _ as *mut _,
                        custom_pbuf.buf.as_ref().unwrap().packet().as_ptr() as *mut _,
                        custom_pbuf.buf.as_ref().unwrap().capacity() as u16,
                    )
                };
                // move to raw pointer to avoid double free
                let _custom_pbuf = Box::into_raw(custom_pbuf);

                debug!("ethernet_input");
                let mut netif = self.netif.lock();
                unsafe {
                    let res = netif.0.input.unwrap()(p, &mut netif.0);
                    if (res as i32) != err_enum_t_ERR_OK {
                        warn!("ethernet_input failed: {:?}", res);
                        pbuf_free(p);
                    }
                }
            } else {
                break;
            }
        }
    }

    #[cfg(feature = "irq")]
    pub fn ack_interrupt(&self) {
        unsafe { &mut *self.dev.as_mut_ptr() }.ack_interrupt();
    }
}

#[repr(C)]
struct CustomPbuf {
    p: pbuf_custom,
    buf: Option<NetBufferBox<'static>>,
    dev: Arc<Mutex<DeviceWrapper>>,
}

extern "C" fn pbuf_free_custom(p: *mut pbuf) {
    debug!("pbuf_free_custom: {:x?}", p);
    let mut custom_pbuf = unsafe { Box::from_raw(p as *mut CustomPbuf) };
    let buf = custom_pbuf.buf.take().unwrap();
    let res = custom_pbuf
        .dev
        .lock()
        .inner
        .borrow_mut()
        .recycle_rx_buffer(buf);
    match res {
        Ok(_) => (),
        Err(err) => {
            warn!("recycle_rx_buffer failed: {:?}", err);
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
        error!("[ethif_output] dev can't transmit");
        err_enum_t_ERR_MEM as err_t
    }
}

static ETH0: LazyInit<InterfaceWrapper> = LazyInit::new();

fn ip4_addr_gen(a: u8, b: u8, c: u8, d: u8) -> ip4_addr_t {
    ip4_addr_t {
        addr: unsafe {
            lwip_htonl(((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | (d as u32))
        },
    }
}

pub fn init(mut net_dev: AxNetDevice) {
    let pool = NetBufferPool::new(NET_BUF_POOL_SIZE, NET_BUF_LEN).unwrap();
    NET_BUF_POOL.init_by(pool);
    net_dev.fill_rx_buffers(&NET_BUF_POOL).unwrap();
    #[cfg(feature = "irq")]
    register_interrupt_handler!(net_dev, {
        info!("ACK");
        ETH0.ack_interrupt();
    });

    LWIP_MUTEX.init_by(Mutex::new(()));
    let _guard = LWIP_MUTEX.lock();

    let ipaddr: ip4_addr_t = ip4_addr_gen(10, 0, 2, 15); // QEMU user networking default IP
    let netmask: ip4_addr_t = ip4_addr_gen(255, 255, 255, 0);
    let gw: ip4_addr_t = ip4_addr_gen(10, 0, 2, 2); // QEMU user networking gateway

    let dev = net_dev;
    let mut netif: netif = unsafe { core::mem::zeroed() };
    netif.hwaddr_len = 6;
    netif.hwaddr = dev.mac_address().0;

    ETH0.init_by(InterfaceWrapper {
        name: "eth0",
        dev: Arc::new(Mutex::new(DeviceWrapper::new(dev))),
        netif: Mutex::new(NetifWrapper(netif)),
    });

    unsafe {
        lwip_init();
        netif_add(
            &mut ETH0.netif.lock().0,
            &ipaddr,
            &netmask,
            &gw,
            &ETH0 as *const _ as *mut c_void,
            Some(ethif_init),
            Some(ethernet_input),
        );
        netif_create_ip6_linklocal_address(&mut ETH0.netif.lock().0, 1);
        netif_set_link_up(&mut ETH0.netif.lock().0);
        netif_set_up(&mut ETH0.netif.lock().0);
        netif_set_default(&mut ETH0.netif.lock().0);
    }

    info!("created net interface {:?}:", ETH0.name());
    info!(
        "  ether:    {}",
        MacAddr::from_bytes(&ETH0.netif.lock().0.hwaddr)
    );
    let ip = IpAddr::from(ETH0.netif.lock().0.ip_addr);
    let mask = mask_to_prefix(IpAddr::from(ETH0.netif.lock().0.netmask)).unwrap();
    info!("  ip:       {}/{}", ip, mask);
    info!("  gateway:  {}", IpAddr::from(ETH0.netif.lock().0.gw));
    info!(
        "  ip6:      {}",
        IpAddr::from(ETH0.netif.lock().0.ip6_addr[0])
    );
}

pub fn lwip_loop_once() {
    let guard = LWIP_MUTEX.lock();
    unsafe {
        ETH0.poll();
        sys_check_timeouts();
    }
    drop(guard);
}
