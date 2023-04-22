mod addr;
mod cbindings;
mod driver;
mod tcp;

pub use addr::{IpAddr, Ipv4Addr, SocketAddr};
pub use driver::init;
pub use tcp::TcpSocket;

use axsync::Mutex;
use lazy_init::LazyInit;

static LWIP_MUTEX: LazyInit<Mutex<()>> = LazyInit::new();
