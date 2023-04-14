use axdriver::NetDevices;
use lwip_rust::bindings::{
    err_enum_t_ERR_OK, err_t, ethernet_input, ip4_addr_t, ip_addr_t, lwip_htonl, lwip_init, netif,
    netif_add, pbuf,
};

fn ip4_addr_gen(a: u8, b: u8, c: u8, d: u8) -> ip4_addr_t {
    ip4_addr_t {
        addr: unsafe {
            lwip_htonl(((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | (d as u32))
                as u32
        },
    }
}

extern "C" fn ethif_init(netif: *mut netif) -> err_t {
    info!("ethif_init");
    err_enum_t_ERR_OK as err_t
}

extern "C" fn ethif_input(netif: *mut netif) -> err_t {
    info!("ethif_input");
    err_enum_t_ERR_OK as err_t
}

extern "C" fn ethif_output(netif: *mut netif, p: *mut pbuf, ipaddr: *const ip_addr_t) -> err_t {
    info!("ethif_output");
    err_enum_t_ERR_OK as err_t
}

pub fn init(_net_devs: NetDevices) {
    let mut netif: netif = unsafe { core::mem::zeroed() };
    let mut ipaddr: ip4_addr_t = ip4_addr_gen(10, 0, 2, 15); // QEMU user networking default IP
    let mut netmask: ip4_addr_t = ip4_addr_gen(255, 255, 255, 0);
    let mut gw: ip4_addr_t = ip4_addr_gen(10, 0, 2, 2); // QEMU user networking gateway
    unsafe {
        lwip_init();
        netif_add(
            &mut netif,
            &mut ipaddr,
            &mut netmask,
            &mut gw,
            core::ptr::null_mut(),
            Some(ethif_init),
            Some(ethernet_input),
        );
        while true {
            ethif_input(&mut netif);
            // sys_check_timeouts();
        }
    }
}
