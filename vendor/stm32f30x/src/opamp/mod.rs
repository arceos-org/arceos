#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - OPAMP1 control register"]
    pub opamp1_cr: OPAMP1_CR,
    #[doc = "0x04 - OPAMP2 control register"]
    pub opamp2_cr: OPAMP2_CR,
    #[doc = "0x08 - OPAMP3 control register"]
    pub opamp3_cr: OPAMP3_CR,
    #[doc = "0x0c - OPAMP4 control register"]
    pub opamp4_cr: OPAMP4_CR,
}
#[doc = "OPAMP1 control register"]
pub struct OPAMP1_CR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "OPAMP1 control register"]
pub mod opamp1_cr;
#[doc = "OPAMP2 control register"]
pub struct OPAMP2_CR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "OPAMP2 control register"]
pub mod opamp2_cr;
#[doc = "OPAMP3 control register"]
pub struct OPAMP3_CR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "OPAMP3 control register"]
pub mod opamp3_cr;
#[doc = "OPAMP4 control register"]
pub struct OPAMP4_CR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "OPAMP4 control register"]
pub mod opamp4_cr;
