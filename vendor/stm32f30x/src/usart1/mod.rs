#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Control register 1"]
    pub cr1: CR1,
    #[doc = "0x04 - Control register 2"]
    pub cr2: CR2,
    #[doc = "0x08 - Control register 3"]
    pub cr3: CR3,
    #[doc = "0x0c - Baud rate register"]
    pub brr: BRR,
    #[doc = "0x10 - Guard time and prescaler register"]
    pub gtpr: GTPR,
    #[doc = "0x14 - Receiver timeout register"]
    pub rtor: RTOR,
    #[doc = "0x18 - Request register"]
    pub rqr: RQR,
    #[doc = "0x1c - Interrupt & status register"]
    pub isr: ISR,
    #[doc = "0x20 - Interrupt flag clear register"]
    pub icr: ICR,
    #[doc = "0x24 - Receive data register"]
    pub rdr: RDR,
    #[doc = "0x28 - Transmit data register"]
    pub tdr: TDR,
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
#[doc = "Control register 3"]
pub struct CR3 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Control register 3"]
pub mod cr3;
#[doc = "Baud rate register"]
pub struct BRR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Baud rate register"]
pub mod brr;
#[doc = "Guard time and prescaler register"]
pub struct GTPR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Guard time and prescaler register"]
pub mod gtpr;
#[doc = "Receiver timeout register"]
pub struct RTOR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Receiver timeout register"]
pub mod rtor;
#[doc = "Request register"]
pub struct RQR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Request register"]
pub mod rqr;
#[doc = "Interrupt & status register"]
pub struct ISR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Interrupt & status register"]
pub mod isr;
#[doc = "Interrupt flag clear register"]
pub struct ICR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Interrupt flag clear register"]
pub mod icr;
#[doc = "Receive data register"]
pub struct RDR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Receive data register"]
pub mod rdr;
#[doc = "Transmit data register"]
pub struct TDR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Transmit data register"]
pub mod tdr;
