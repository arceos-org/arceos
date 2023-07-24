#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - control register 1"]
    pub cr1: CR1,
    #[doc = "0x04 - control register 2"]
    pub cr2: CR2,
    #[doc = "0x08 - status register"]
    pub sr: SR,
    #[doc = "0x0c - data register"]
    pub dr: DR,
    #[doc = "0x10 - CRC polynomial register"]
    pub crcpr: CRCPR,
    #[doc = "0x14 - RX CRC register"]
    pub rxcrcr: RXCRCR,
    #[doc = "0x18 - TX CRC register"]
    pub txcrcr: TXCRCR,
    #[doc = "0x1c - I2S configuration register"]
    pub i2scfgr: I2SCFGR,
    #[doc = "0x20 - I2S prescaler register"]
    pub i2spr: I2SPR,
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
#[doc = "status register"]
pub struct SR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "status register"]
pub mod sr;
#[doc = "data register"]
pub struct DR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "data register"]
pub mod dr;
#[doc = "CRC polynomial register"]
pub struct CRCPR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "CRC polynomial register"]
pub mod crcpr;
#[doc = "RX CRC register"]
pub struct RXCRCR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "RX CRC register"]
pub mod rxcrcr;
#[doc = "TX CRC register"]
pub struct TXCRCR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "TX CRC register"]
pub mod txcrcr;
#[doc = "I2S configuration register"]
pub struct I2SCFGR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "I2S configuration register"]
pub mod i2scfgr;
#[doc = "I2S prescaler register"]
pub struct I2SPR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "I2S prescaler register"]
pub mod i2spr;
