use super::driver::lwip_loop_once;
use crate::{IpAddr, Ipv4Addr};
use alloc::{ffi::CString, vec, vec::Vec};
use axerrno::{ax_err, AxResult};
use axtask::yield_now;
use core::{
    ffi::{c_char, c_void, CStr},
    str::FromStr,
};
use lwip_rust::bindings::{
    dns_gethostbyname, dns_setserver, err_enum_t_ERR_ARG, err_enum_t_ERR_INPROGRESS,
    err_enum_t_ERR_OK, err_enum_t_ERR_VAL, ip_addr_t,
};

use super::LWIP_MUTEX;

struct DnsQueryEntry {
    ipaddr: Option<IpAddr>,
    finished: bool,
}

extern "C" fn dns_found_callback(
    name: *const c_char,
    ipaddr: *const ip_addr_t,
    callback_arg: *mut c_void,
) {
    trace!(
        "[dns_found_callback]: name_ptr={:?} ipaddr_ptr={:?}",
        name,
        ipaddr
    );
    let res = callback_arg as *mut DnsQueryEntry;
    unsafe {
        (*res).finished = true;
        (*res).ipaddr = if ipaddr.is_null() {
            None
        } else {
            debug!(
                "DNS found: name={} ipaddr={}",
                CStr::from_ptr(name as *mut c_char).to_str().unwrap(),
                IpAddr::from(*ipaddr)
            );
            Some((*ipaddr).into())
        };
    }
}

pub fn resolve_socket_addr(name: &str) -> AxResult<Vec<IpAddr>> {
    let guard = LWIP_MUTEX.lock();
    unsafe {
        dns_setserver(
            0,
            &IpAddr::from_str("8.8.8.8").unwrap().into() as *const ip_addr_t,
        )
    };

    let mut addr: ip_addr_t = IpAddr::Ipv4(Ipv4Addr(0)).into();
    let mut query_entry = DnsQueryEntry {
        ipaddr: None,
        finished: false,
    };
    let name = CString::new(name).unwrap();
    let res = unsafe {
        dns_gethostbyname(
            name.as_ptr(),
            &mut addr as *mut ip_addr_t,
            Some(dns_found_callback),
            &mut query_entry as *mut DnsQueryEntry as *mut c_void,
        ) as i32
    };
    drop(guard);

    #[allow(non_upper_case_globals)]
    match res {
        err_enum_t_ERR_OK => Ok(vec![addr.into()]),
        err_enum_t_ERR_INPROGRESS => loop {
            lwip_loop_once();
            if query_entry.finished {
                break if query_entry.ipaddr.is_some() {
                    Ok(vec![query_entry.ipaddr.unwrap()])
                } else {
                    ax_err!(NotFound, "LWIP dns not found")
                };
            }
            yield_now();
        },
        err_enum_t_ERR_ARG => ax_err!(
            InvalidInput,
            "LWIP dns client not initialized or invalid hostname"
        ),
        err_enum_t_ERR_VAL => ax_err!(
            InvalidInput,
            "LWIP dns client error, perhaps dns server not configured"
        ),
        _ => ax_err!(InvalidInput, "LWIP dns client error"),
    }
}
