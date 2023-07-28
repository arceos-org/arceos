#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - power control register"]
    pub cr: CR,
    #[doc = "0x04 - power control/status register"]
    pub csr: CSR,
}
#[doc = "power control register"]
pub struct CR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "power control register"]
pub mod cr;
#[doc = "power control/status register"]
pub struct CSR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "power control/status register"]
pub mod csr;
