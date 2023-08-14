#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - configuration register 1"]
    pub cfgr1: CFGR1,
    #[doc = "0x04 - CCM SRAM protection register"]
    pub rcr: RCR,
    #[doc = "0x08 - external interrupt configuration register 1"]
    pub exticr1: EXTICR1,
    #[doc = "0x0c - external interrupt configuration register 2"]
    pub exticr2: EXTICR2,
    #[doc = "0x10 - external interrupt configuration register 3"]
    pub exticr3: EXTICR3,
    #[doc = "0x14 - external interrupt configuration register 4"]
    pub exticr4: EXTICR4,
    #[doc = "0x18 - configuration register 2"]
    pub cfgr2: CFGR2,
}
#[doc = "configuration register 1"]
pub struct CFGR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "configuration register 1"]
pub mod cfgr1;
#[doc = "external interrupt configuration register 1"]
pub struct EXTICR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "external interrupt configuration register 1"]
pub mod exticr1;
#[doc = "external interrupt configuration register 2"]
pub struct EXTICR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "external interrupt configuration register 2"]
pub mod exticr2;
#[doc = "external interrupt configuration register 3"]
pub struct EXTICR3 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "external interrupt configuration register 3"]
pub mod exticr3;
#[doc = "external interrupt configuration register 4"]
pub struct EXTICR4 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "external interrupt configuration register 4"]
pub mod exticr4;
#[doc = "configuration register 2"]
pub struct CFGR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "configuration register 2"]
pub mod cfgr2;
#[doc = "CCM SRAM protection register"]
pub struct RCR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "CCM SRAM protection register"]
pub mod rcr;
