#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Data register"]
    pub dr: DR,
    #[doc = "0x04 - Independent data register"]
    pub idr: IDR,
    #[doc = "0x08 - Control register"]
    pub cr: CR,
    _reserved0: [u8; 4usize],
    #[doc = "0x10 - Initial CRC value"]
    pub init: INIT,
    #[doc = "0x14 - CRC polynomial"]
    pub pol: POL,
}
#[doc = "Data register"]
pub struct DR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Data register"]
pub mod dr;
#[doc = "Independent data register"]
pub struct IDR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Independent data register"]
pub mod idr;
#[doc = "Control register"]
pub struct CR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Control register"]
pub mod cr;
#[doc = "Initial CRC value"]
pub struct INIT {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Initial CRC value"]
pub mod init;
#[doc = "CRC polynomial"]
pub struct POL {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "CRC polynomial"]
pub mod pol;
