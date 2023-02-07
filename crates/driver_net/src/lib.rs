#![no_std]

#[derive(Debug)]
pub enum NetDevError {
    Dummy,
}

pub type NetDevResult<T = ()> = Result<T, NetDevError>;

pub trait NetDriverOps: Send + Sync {
    fn send(&self, buf: &[u8]) -> NetDevResult<usize>;
    fn recv(&self, buf: &mut [u8]) -> NetDevResult<usize>;
}
