#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - GPIO port mode register"]
    pub moder: MODER,
    #[doc = "0x04 - GPIO port output type register"]
    pub otyper: OTYPER,
    #[doc = "0x08 - GPIO port output speed register"]
    pub ospeedr: OSPEEDR,
    #[doc = "0x0c - GPIO port pull-up/pull-down register"]
    pub pupdr: PUPDR,
    #[doc = "0x10 - GPIO port input data register"]
    pub idr: IDR,
    #[doc = "0x14 - GPIO port output data register"]
    pub odr: ODR,
    #[doc = "0x18 - GPIO port bit set/reset register"]
    pub bsrr: BSRR,
    #[doc = "0x1c - GPIO port configuration lock register"]
    pub lckr: LCKR,
    #[doc = "0x20 - GPIO alternate function low register"]
    pub afrl: AFRL,
    #[doc = "0x24 - GPIO alternate function high register"]
    pub afrh: AFRH,
    #[doc = "0x28 - Port bit reset register"]
    pub brr: BRR,
}
#[doc = "GPIO port mode register"]
pub struct MODER {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "GPIO port mode register"]
pub mod moder;
#[doc = "GPIO port output type register"]
pub struct OTYPER {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "GPIO port output type register"]
pub mod otyper;
#[doc = "GPIO port output speed register"]
pub struct OSPEEDR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "GPIO port output speed register"]
pub mod ospeedr;
#[doc = "GPIO port pull-up/pull-down register"]
pub struct PUPDR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "GPIO port pull-up/pull-down register"]
pub mod pupdr;
#[doc = "GPIO port input data register"]
pub struct IDR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "GPIO port input data register"]
pub mod idr;
#[doc = "GPIO port output data register"]
pub struct ODR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "GPIO port output data register"]
pub mod odr;
#[doc = "GPIO port bit set/reset register"]
pub struct BSRR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "GPIO port bit set/reset register"]
pub mod bsrr;
#[doc = "GPIO port configuration lock register"]
pub struct LCKR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "GPIO port configuration lock register"]
pub mod lckr;
#[doc = "GPIO alternate function low register"]
pub struct AFRL {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "GPIO alternate function low register"]
pub mod afrl;
#[doc = "GPIO alternate function high register"]
pub struct AFRH {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "GPIO alternate function high register"]
pub mod afrh;
#[doc = "Port bit reset register"]
pub struct BRR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Port bit reset register"]
pub mod brr;
