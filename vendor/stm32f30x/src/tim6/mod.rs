#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - control register 1"]
    pub cr1: CR1,
    #[doc = "0x04 - control register 2"]
    pub cr2: CR2,
    _reserved0: [u8; 4usize],
    #[doc = "0x0c - DMA/Interrupt enable register"]
    pub dier: DIER,
    #[doc = "0x10 - status register"]
    pub sr: SR,
    #[doc = "0x14 - event generation register"]
    pub egr: EGR,
    _reserved1: [u8; 12usize],
    #[doc = "0x24 - counter"]
    pub cnt: CNT,
    #[doc = "0x28 - prescaler"]
    pub psc: PSC,
    #[doc = "0x2c - auto-reload register"]
    pub arr: ARR,
}
#[doc = "control register 1"]
pub struct CR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "control register 1"]
pub mod cr1;
#[doc = "control register 2"]
pub struct CR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "control register 2"]
pub mod cr2;
#[doc = "DMA/Interrupt enable register"]
pub struct DIER {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA/Interrupt enable register"]
pub mod dier;
#[doc = "status register"]
pub struct SR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "status register"]
pub mod sr;
#[doc = "event generation register"]
pub struct EGR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "event generation register"]
pub mod egr;
#[doc = "counter"]
pub struct CNT {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "counter"]
pub mod cnt;
#[doc = "prescaler"]
pub struct PSC {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "prescaler"]
pub mod psc;
#[doc = "auto-reload register"]
pub struct ARR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "auto-reload register"]
pub mod arr;
