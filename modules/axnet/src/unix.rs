use alloc::sync::Arc;

#[derive(Clone, Debug)]
pub enum UnixSocketAddr {
    Unnamed,
    Abstract(Arc<[u8]>),
    Path(Arc<str>),
}
