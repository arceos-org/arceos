#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - DMA interrupt status register (DMA_ISR)"]
    pub isr: ISR,
    #[doc = "0x04 - DMA interrupt flag clear register (DMA_IFCR)"]
    pub ifcr: IFCR,
    #[doc = "0x08 - DMA channel configuration register (DMA_CCR)"]
    pub ccr1: CCR1,
    #[doc = "0x0c - DMA channel 1 number of data register"]
    pub cndtr1: CNDTR1,
    #[doc = "0x10 - DMA channel 1 peripheral address register"]
    pub cpar1: CPAR1,
    #[doc = "0x14 - DMA channel 1 memory address register"]
    pub cmar1: CMAR1,
    _reserved0: [u8; 4usize],
    #[doc = "0x1c - DMA channel configuration register (DMA_CCR)"]
    pub ccr2: CCR2,
    #[doc = "0x20 - DMA channel 2 number of data register"]
    pub cndtr2: CNDTR2,
    #[doc = "0x24 - DMA channel 2 peripheral address register"]
    pub cpar2: CPAR2,
    #[doc = "0x28 - DMA channel 2 memory address register"]
    pub cmar2: CMAR2,
    _reserved1: [u8; 4usize],
    #[doc = "0x30 - DMA channel configuration register (DMA_CCR)"]
    pub ccr3: CCR3,
    #[doc = "0x34 - DMA channel 3 number of data register"]
    pub cndtr3: CNDTR3,
    #[doc = "0x38 - DMA channel 3 peripheral address register"]
    pub cpar3: CPAR3,
    #[doc = "0x3c - DMA channel 3 memory address register"]
    pub cmar3: CMAR3,
    _reserved2: [u8; 4usize],
    #[doc = "0x44 - DMA channel configuration register (DMA_CCR)"]
    pub ccr4: CCR4,
    #[doc = "0x48 - DMA channel 4 number of data register"]
    pub cndtr4: CNDTR4,
    #[doc = "0x4c - DMA channel 4 peripheral address register"]
    pub cpar4: CPAR4,
    #[doc = "0x50 - DMA channel 4 memory address register"]
    pub cmar4: CMAR4,
    _reserved3: [u8; 4usize],
    #[doc = "0x58 - DMA channel configuration register (DMA_CCR)"]
    pub ccr5: CCR5,
    #[doc = "0x5c - DMA channel 5 number of data register"]
    pub cndtr5: CNDTR5,
    #[doc = "0x60 - DMA channel 5 peripheral address register"]
    pub cpar5: CPAR5,
    #[doc = "0x64 - DMA channel 5 memory address register"]
    pub cmar5: CMAR5,
    _reserved4: [u8; 4usize],
    #[doc = "0x6c - DMA channel configuration register (DMA_CCR)"]
    pub ccr6: CCR6,
    #[doc = "0x70 - DMA channel 6 number of data register"]
    pub cndtr6: CNDTR6,
    #[doc = "0x74 - DMA channel 6 peripheral address register"]
    pub cpar6: CPAR6,
    #[doc = "0x78 - DMA channel 6 memory address register"]
    pub cmar6: CMAR6,
    _reserved5: [u8; 4usize],
    #[doc = "0x80 - DMA channel configuration register (DMA_CCR)"]
    pub ccr7: CCR7,
    #[doc = "0x84 - DMA channel 7 number of data register"]
    pub cndtr7: CNDTR7,
    #[doc = "0x88 - DMA channel 7 peripheral address register"]
    pub cpar7: CPAR7,
    #[doc = "0x8c - DMA channel 7 memory address register"]
    pub cmar7: CMAR7,
}
#[doc = "DMA interrupt status register (DMA_ISR)"]
pub struct ISR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA interrupt status register (DMA_ISR)"]
pub mod isr;
#[doc = "DMA interrupt flag clear register (DMA_IFCR)"]
pub struct IFCR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA interrupt flag clear register (DMA_IFCR)"]
pub mod ifcr;
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub struct CCR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub mod ccr1;
#[doc = "DMA channel 1 number of data register"]
pub struct CNDTR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 1 number of data register"]
pub mod cndtr1;
#[doc = "DMA channel 1 peripheral address register"]
pub struct CPAR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 1 peripheral address register"]
pub mod cpar1;
#[doc = "DMA channel 1 memory address register"]
pub struct CMAR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 1 memory address register"]
pub mod cmar1;
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub struct CCR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub mod ccr2;
#[doc = "DMA channel 2 number of data register"]
pub struct CNDTR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 2 number of data register"]
pub mod cndtr2;
#[doc = "DMA channel 2 peripheral address register"]
pub struct CPAR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 2 peripheral address register"]
pub mod cpar2;
#[doc = "DMA channel 2 memory address register"]
pub struct CMAR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 2 memory address register"]
pub mod cmar2;
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub struct CCR3 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub mod ccr3;
#[doc = "DMA channel 3 number of data register"]
pub struct CNDTR3 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 3 number of data register"]
pub mod cndtr3;
#[doc = "DMA channel 3 peripheral address register"]
pub struct CPAR3 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 3 peripheral address register"]
pub mod cpar3;
#[doc = "DMA channel 3 memory address register"]
pub struct CMAR3 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 3 memory address register"]
pub mod cmar3;
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub struct CCR4 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub mod ccr4;
#[doc = "DMA channel 4 number of data register"]
pub struct CNDTR4 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 4 number of data register"]
pub mod cndtr4;
#[doc = "DMA channel 4 peripheral address register"]
pub struct CPAR4 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 4 peripheral address register"]
pub mod cpar4;
#[doc = "DMA channel 4 memory address register"]
pub struct CMAR4 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 4 memory address register"]
pub mod cmar4;
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub struct CCR5 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub mod ccr5;
#[doc = "DMA channel 5 number of data register"]
pub struct CNDTR5 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 5 number of data register"]
pub mod cndtr5;
#[doc = "DMA channel 5 peripheral address register"]
pub struct CPAR5 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 5 peripheral address register"]
pub mod cpar5;
#[doc = "DMA channel 5 memory address register"]
pub struct CMAR5 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 5 memory address register"]
pub mod cmar5;
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub struct CCR6 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub mod ccr6;
#[doc = "DMA channel 6 number of data register"]
pub struct CNDTR6 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 6 number of data register"]
pub mod cndtr6;
#[doc = "DMA channel 6 peripheral address register"]
pub struct CPAR6 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 6 peripheral address register"]
pub mod cpar6;
#[doc = "DMA channel 6 memory address register"]
pub struct CMAR6 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 6 memory address register"]
pub mod cmar6;
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub struct CCR7 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel configuration register (DMA_CCR)"]
pub mod ccr7;
#[doc = "DMA channel 7 number of data register"]
pub struct CNDTR7 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 7 number of data register"]
pub mod cndtr7;
#[doc = "DMA channel 7 peripheral address register"]
pub struct CPAR7 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 7 peripheral address register"]
pub mod cpar7;
#[doc = "DMA channel 7 memory address register"]
pub struct CMAR7 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA channel 7 memory address register"]
pub mod cmar7;
