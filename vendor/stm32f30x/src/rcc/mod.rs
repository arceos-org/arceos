#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Clock control register"]
    pub cr: CR,
    #[doc = "0x04 - Clock configuration register (RCC_CFGR)"]
    pub cfgr: CFGR,
    #[doc = "0x08 - Clock interrupt register (RCC_CIR)"]
    pub cir: CIR,
    #[doc = "0x0c - APB2 peripheral reset register (RCC_APB2RSTR)"]
    pub apb2rstr: APB2RSTR,
    #[doc = "0x10 - APB1 peripheral reset register (RCC_APB1RSTR)"]
    pub apb1rstr: APB1RSTR,
    #[doc = "0x14 - AHB Peripheral Clock enable register (RCC_AHBENR)"]
    pub ahbenr: AHBENR,
    #[doc = "0x18 - APB2 peripheral clock enable register (RCC_APB2ENR)"]
    pub apb2enr: APB2ENR,
    #[doc = "0x1c - APB1 peripheral clock enable register (RCC_APB1ENR)"]
    pub apb1enr: APB1ENR,
    #[doc = "0x20 - Backup domain control register (RCC_BDCR)"]
    pub bdcr: BDCR,
    #[doc = "0x24 - Control/status register (RCC_CSR)"]
    pub csr: CSR,
    #[doc = "0x28 - AHB peripheral reset register"]
    pub ahbrstr: AHBRSTR,
    #[doc = "0x2c - Clock configuration register 2"]
    pub cfgr2: CFGR2,
    #[doc = "0x30 - Clock configuration register 3"]
    pub cfgr3: CFGR3,
}
#[doc = "Clock control register"]
pub struct CR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Clock control register"]
pub mod cr;
#[doc = "Clock configuration register (RCC_CFGR)"]
pub struct CFGR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Clock configuration register (RCC_CFGR)"]
pub mod cfgr;
#[doc = "Clock interrupt register (RCC_CIR)"]
pub struct CIR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Clock interrupt register (RCC_CIR)"]
pub mod cir;
#[doc = "APB2 peripheral reset register (RCC_APB2RSTR)"]
pub struct APB2RSTR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "APB2 peripheral reset register (RCC_APB2RSTR)"]
pub mod apb2rstr;
#[doc = "APB1 peripheral reset register (RCC_APB1RSTR)"]
pub struct APB1RSTR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "APB1 peripheral reset register (RCC_APB1RSTR)"]
pub mod apb1rstr;
#[doc = "AHB Peripheral Clock enable register (RCC_AHBENR)"]
pub struct AHBENR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "AHB Peripheral Clock enable register (RCC_AHBENR)"]
pub mod ahbenr;
#[doc = "APB2 peripheral clock enable register (RCC_APB2ENR)"]
pub struct APB2ENR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "APB2 peripheral clock enable register (RCC_APB2ENR)"]
pub mod apb2enr;
#[doc = "APB1 peripheral clock enable register (RCC_APB1ENR)"]
pub struct APB1ENR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "APB1 peripheral clock enable register (RCC_APB1ENR)"]
pub mod apb1enr;
#[doc = "Backup domain control register (RCC_BDCR)"]
pub struct BDCR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Backup domain control register (RCC_BDCR)"]
pub mod bdcr;
#[doc = "Control/status register (RCC_CSR)"]
pub struct CSR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Control/status register (RCC_CSR)"]
pub mod csr;
#[doc = "AHB peripheral reset register"]
pub struct AHBRSTR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "AHB peripheral reset register"]
pub mod ahbrstr;
#[doc = "Clock configuration register 2"]
pub struct CFGR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Clock configuration register 2"]
pub mod cfgr2;
#[doc = "Clock configuration register 3"]
pub struct CFGR3 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Clock configuration register 3"]
pub mod cfgr3;
