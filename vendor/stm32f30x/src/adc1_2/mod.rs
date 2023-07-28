#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - ADC Common status register"]
    pub csr: CSR,
    _reserved0: [u8; 4usize],
    #[doc = "0x08 - ADC common control register"]
    pub ccr: CCR,
    #[doc = "0x0c - ADC common regular data register for dual and triple modes"]
    pub cdr: CDR,
}
#[doc = "ADC Common status register"]
pub struct CSR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "ADC Common status register"]
pub mod csr;
#[doc = "ADC common control register"]
pub struct CCR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "ADC common control register"]
pub mod ccr;
#[doc = "ADC common regular data register for dual and triple modes"]
pub struct CDR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "ADC common regular data register for dual and triple modes"]
pub mod cdr;
