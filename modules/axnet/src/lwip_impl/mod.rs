mod addr;
mod cbindings;
mod dns;
mod driver;
mod tcp;
mod udp;

pub use self::addr::{IpAddr, Ipv4Addr, SocketAddr};
pub use self::dns::resolve_socket_addr;
pub use self::driver::init;
pub use self::tcp::TcpSocket;
pub use self::udp::UdpSocket;

use axsync::Mutex;
use lazy_init::LazyInit;

static LWIP_MUTEX: LazyInit<Mutex<()>> = LazyInit::new();

const RECV_QUEUE_LEN: usize = 16;
const ACCEPT_QUEUE_LEN: usize = 16;
