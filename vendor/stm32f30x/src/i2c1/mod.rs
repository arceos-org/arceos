#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Control register 1"]
    pub cr1: CR1,
    #[doc = "0x04 - Control register 2"]
    pub cr2: CR2,
    #[doc = "0x08 - Own address register 1"]
    pub oar1: OAR1,
    #[doc = "0x0c - Own address register 2"]
    pub oar2: OAR2,
    #[doc = "0x10 - Timing register"]
    pub timingr: TIMINGR,
    #[doc = "0x14 - Status register 1"]
    pub timeoutr: TIMEOUTR,
    #[doc = "0x18 - Interrupt and Status register"]
    pub isr: ISR,
    #[doc = "0x1c - Interrupt clear register"]
    pub icr: ICR,
    #[doc = "0x20 - PEC register"]
    pub pecr: PECR,
    #[doc = "0x24 - Receive data register"]
    pub rxdr: RXDR,
    #[doc = "0x28 - Transmit data register"]
    pub txdr: TXDR,
}
#[doc = "Control register 1"]
pub struct CR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Control register 1"]
pub mod cr1;
#[doc = "Control register 2"]
pub struct CR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Control register 2"]
pub mod cr2;
#[doc = "Own address register 1"]
pub struct OAR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Own address register 1"]
pub mod oar1;
#[doc = "Own address register 2"]
pub struct OAR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Own address register 2"]
pub mod oar2;
#[doc = "Timing register"]
pub struct TIMINGR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Timing register"]
pub mod timingr;
#[doc = "Status register 1"]
pub struct TIMEOUTR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Status register 1"]
pub mod timeoutr;
#[doc = "Interrupt and Status register"]
pub struct ISR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Interrupt and Status register"]
pub mod isr;
#[doc = "Interrupt clear register"]
pub struct ICR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Interrupt clear register"]
pub mod icr;
#[doc = "PEC register"]
pub struct PECR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "PEC register"]
pub mod pecr;
#[doc = "Receive data register"]
pub struct RXDR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Receive data register"]
pub mod rxdr;
#[doc = "Transmit data register"]
pub struct TXDR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Transmit data register"]
pub mod txdr;
