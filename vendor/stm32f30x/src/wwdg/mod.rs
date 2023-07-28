#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Control register"]
    pub cr: CR,
    #[doc = "0x04 - Configuration register"]
    pub cfr: CFR,
    #[doc = "0x08 - Status register"]
    pub sr: SR,
}
#[doc = "Control register"]
pub struct CR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Control register"]
pub mod cr;
#[doc = "Configuration register"]
pub struct CFR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Configuration register"]
pub mod cfr;
#[doc = "Status register"]
pub struct SR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Status register"]
pub mod sr;
