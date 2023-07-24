#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Interrupt mask register"]
    pub imr1: IMR1,
    #[doc = "0x04 - Event mask register"]
    pub emr1: EMR1,
    #[doc = "0x08 - Rising Trigger selection register"]
    pub rtsr1: RTSR1,
    #[doc = "0x0c - Falling Trigger selection register"]
    pub ftsr1: FTSR1,
    #[doc = "0x10 - Software interrupt event register"]
    pub swier1: SWIER1,
    #[doc = "0x14 - Pending register"]
    pub pr1: PR1,
    #[doc = "0x18 - Interrupt mask register"]
    pub imr2: IMR2,
    #[doc = "0x1c - Event mask register"]
    pub emr2: EMR2,
    #[doc = "0x20 - Rising Trigger selection register"]
    pub rtsr2: RTSR2,
    #[doc = "0x24 - Falling Trigger selection register"]
    pub ftsr2: FTSR2,
    #[doc = "0x28 - Software interrupt event register"]
    pub swier2: SWIER2,
    #[doc = "0x2c - Pending register"]
    pub pr2: PR2,
}
#[doc = "Interrupt mask register"]
pub struct IMR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Interrupt mask register"]
pub mod imr1;
#[doc = "Event mask register"]
pub struct EMR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Event mask register"]
pub mod emr1;
#[doc = "Rising Trigger selection register"]
pub struct RTSR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Rising Trigger selection register"]
pub mod rtsr1;
#[doc = "Falling Trigger selection register"]
pub struct FTSR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Falling Trigger selection register"]
pub mod ftsr1;
#[doc = "Software interrupt event register"]
pub struct SWIER1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Software interrupt event register"]
pub mod swier1;
#[doc = "Pending register"]
pub struct PR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Pending register"]
pub mod pr1;
#[doc = "Interrupt mask register"]
pub struct IMR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Interrupt mask register"]
pub mod imr2;
#[doc = "Event mask register"]
pub struct EMR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Event mask register"]
pub mod emr2;
#[doc = "Rising Trigger selection register"]
pub struct RTSR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Rising Trigger selection register"]
pub mod rtsr2;
#[doc = "Falling Trigger selection register"]
pub struct FTSR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Falling Trigger selection register"]
pub mod ftsr2;
#[doc = "Software interrupt event register"]
pub struct SWIER2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Software interrupt event register"]
pub mod swier2;
#[doc = "Pending register"]
pub struct PR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Pending register"]
pub mod pr2;
