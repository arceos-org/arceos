use alloc::boxed::Box;
use axdriver::NetDevices;
use driver_net::NetDriverOps;
use lwip_rust::bindings::{
    err_enum_t_ERR_OK, err_t, etharp_output, ethernet_input, ethip6_output, ip4_addr_t, ip_addr_t,
    lwip_htonl, lwip_init, netif, netif_add, netif_set_default, netif_set_link_up, netif_set_up,
    pbuf, NETIF_FLAG_BROADCAST, NETIF_FLAG_ETHARP, NETIF_FLAG_ETHERNET,
};

fn ip4_addr_gen(a: u8, b: u8, c: u8, d: u8) -> ip4_addr_t {
    ip4_addr_t {
        addr: unsafe {
            lwip_htonl(((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | (d as u32))
                as u32
        },
    }
}

fn mac_addr_gen(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> [u8; 6] {
    [a, b, c, d, e, f]
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

extern "C" fn ethif_input(netif: *mut netif) -> err_t {
    info!("ethif_input");
    err_enum_t_ERR_OK as err_t
}

extern "C" fn ethif_output(netif: *mut netif, p: *mut pbuf) -> err_t {
    info!("ethif_output");
    err_enum_t_ERR_OK as err_t
}

pub fn init(net_devs: NetDevices) {
    let mut ipaddr: ip4_addr_t = ip4_addr_gen(10, 0, 2, 15); // QEMU user networking default IP
    let mut netmask: ip4_addr_t = ip4_addr_gen(255, 255, 255, 0);
    let mut gw: ip4_addr_t = ip4_addr_gen(10, 0, 2, 2); // QEMU user networking gateway

    let dev = net_devs.0;
    let mut netif: Box<netif> = unsafe { Box::new_zeroed().assume_init() };
    netif.hwaddr_len = 6;
    netif.hwaddr = dev.mac_address().0;

    unsafe {
        lwip_init();
        netif_add(
            &mut *netif,
            &mut ipaddr,
            &mut netmask,
            &mut gw,
            core::ptr::null_mut(),
            Some(ethif_init),
            Some(ethernet_input),
        );
        netif_set_link_up(&mut *netif);
        netif_set_up(&mut *netif);
        netif_set_default(&mut *netif);

        // while true {
        //     ethif_input(&mut *netif);
        //     sys_check_timeouts();
        // }
    }
    Box::into_raw(netif);
}
