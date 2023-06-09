#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - interrupt and status register"]
    pub isr: ISR,
    #[doc = "0x04 - interrupt enable register"]
    pub ier: IER,
    #[doc = "0x08 - control register"]
    pub cr: CR,
    #[doc = "0x0c - configuration register"]
    pub cfgr: CFGR,
    _reserved0: [u8; 4usize],
    #[doc = "0x14 - sample time register 1"]
    pub smpr1: SMPR1,
    #[doc = "0x18 - sample time register 2"]
    pub smpr2: SMPR2,
    _reserved1: [u8; 4usize],
    #[doc = "0x20 - watchdog threshold register 1"]
    pub tr1: TR1,
    #[doc = "0x24 - watchdog threshold register"]
    pub tr2: TR2,
    #[doc = "0x28 - watchdog threshold register 3"]
    pub tr3: TR3,
    _reserved2: [u8; 4usize],
    #[doc = "0x30 - regular sequence register 1"]
    pub sqr1: SQR1,
    #[doc = "0x34 - regular sequence register 2"]
    pub sqr2: SQR2,
    #[doc = "0x38 - regular sequence register 3"]
    pub sqr3: SQR3,
    #[doc = "0x3c - regular sequence register 4"]
    pub sqr4: SQR4,
    #[doc = "0x40 - regular Data Register"]
    pub dr: DR,
    _reserved3: [u8; 8usize],
    #[doc = "0x4c - injected sequence register"]
    pub jsqr: JSQR,
    _reserved4: [u8; 16usize],
    #[doc = "0x60 - offset register 1"]
    pub ofr1: OFR1,
    #[doc = "0x64 - offset register 2"]
    pub ofr2: OFR2,
    #[doc = "0x68 - offset register 3"]
    pub ofr3: OFR3,
    #[doc = "0x6c - offset register 4"]
    pub ofr4: OFR4,
    _reserved5: [u8; 16usize],
    #[doc = "0x80 - injected data register 1"]
    pub jdr1: JDR1,
    #[doc = "0x84 - injected data register 2"]
    pub jdr2: JDR2,
    #[doc = "0x88 - injected data register 3"]
    pub jdr3: JDR3,
    #[doc = "0x8c - injected data register 4"]
    pub jdr4: JDR4,
    _reserved6: [u8; 16usize],
    #[doc = "0xa0 - Analog Watchdog 2 Configuration Register"]
    pub awd2cr: AWD2CR,
    #[doc = "0xa4 - Analog Watchdog 3 Configuration Register"]
    pub awd3cr: AWD3CR,
    _reserved7: [u8; 8usize],
    #[doc = "0xb0 - Differential Mode Selection Register 2"]
    pub difsel: DIFSEL,
    #[doc = "0xb4 - Calibration Factors"]
    pub calfact: CALFACT,
}
#[doc = "interrupt and status register"]
pub struct ISR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "interrupt and status register"]
pub mod isr;
#[doc = "interrupt enable register"]
pub struct IER {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "interrupt enable register"]
pub mod ier;
#[doc = "control register"]
pub struct CR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "control register"]
pub mod cr;
#[doc = "configuration register"]
pub struct CFGR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "configuration register"]
pub mod cfgr;
#[doc = "sample time register 1"]
pub struct SMPR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "sample time register 1"]
pub mod smpr1;
#[doc = "sample time register 2"]
pub struct SMPR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "sample time register 2"]
pub mod smpr2;
#[doc = "watchdog threshold register 1"]
pub struct TR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "watchdog threshold register 1"]
pub mod tr1;
#[doc = "watchdog threshold register"]
pub struct TR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "watchdog threshold register"]
pub mod tr2;
#[doc = "watchdog threshold register 3"]
pub struct TR3 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "watchdog threshold register 3"]
pub mod tr3;
#[doc = "regular sequence register 1"]
pub struct SQR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "regular sequence register 1"]
pub mod sqr1;
#[doc = "regular sequence register 2"]
pub struct SQR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "regular sequence register 2"]
pub mod sqr2;
#[doc = "regular sequence register 3"]
pub struct SQR3 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "regular sequence register 3"]
pub mod sqr3;
#[doc = "regular sequence register 4"]
pub struct SQR4 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "regular sequence register 4"]
pub mod sqr4;
#[doc = "regular Data Register"]
pub struct DR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "regular Data Register"]
pub mod dr;
#[doc = "injected sequence register"]
pub struct JSQR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "injected sequence register"]
pub mod jsqr;
#[doc = "offset register 1"]
pub struct OFR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "offset register 1"]
pub mod ofr1;
#[doc = "offset register 2"]
pub struct OFR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "offset register 2"]
pub mod ofr2;
#[doc = "offset register 3"]
pub struct OFR3 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "offset register 3"]
pub mod ofr3;
#[doc = "offset register 4"]
pub struct OFR4 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "offset register 4"]
pub mod ofr4;
#[doc = "injected data register 1"]
pub struct JDR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "injected data register 1"]
pub mod jdr1;
#[doc = "injected data register 2"]
pub struct JDR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "injected data register 2"]
pub mod jdr2;
#[doc = "injected data register 3"]
pub struct JDR3 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "injected data register 3"]
pub mod jdr3;
#[doc = "injected data register 4"]
pub struct JDR4 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "injected data register 4"]
pub mod jdr4;
#[doc = "Analog Watchdog 2 Configuration Register"]
pub struct AWD2CR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Analog Watchdog 2 Configuration Register"]
pub mod awd2cr;
#[doc = "Analog Watchdog 3 Configuration Register"]
pub struct AWD3CR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Analog Watchdog 3 Configuration Register"]
pub mod awd3cr;
#[doc = "Differential Mode Selection Register 2"]
pub struct DIFSEL {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Differential Mode Selection Register 2"]
pub mod difsel;
#[doc = "Calibration Factors"]
pub struct CALFACT {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Calibration Factors"]
pub mod calfact;
