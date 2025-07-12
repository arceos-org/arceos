use smoltcp::wire::{IpAddress, IpEndpoint};

macro_rules! env_or_default {
    ($key:literal) => {
        match option_env!($key) {
            Some(val) => val,
            None => "",
        }
    };
}

pub const IP: &str = env_or_default!("AX_IP");
pub const GATEWAY: &str = env_or_default!("AX_GW");
pub const DNS_SEVER: &str = "8.8.8.8";
pub const IP_PREFIX: u8 = 24;

pub const STANDARD_MTU: usize = 1500;

pub const RANDOM_SEED: u64 = 0xA2CE_05A2_CE05_A2CE;

pub const TCP_RX_BUF_LEN: usize = 64 * 1024;
pub const TCP_TX_BUF_LEN: usize = 64 * 1024;
pub const UDP_RX_BUF_LEN: usize = 64 * 1024;
pub const UDP_TX_BUF_LEN: usize = 64 * 1024;
pub const LISTEN_QUEUE_SIZE: usize = 512;

pub const UNSPECIFIED_ENDPOINT_V4: IpEndpoint = IpEndpoint::new(IpAddress::v4(0, 0, 0, 0), 0);
