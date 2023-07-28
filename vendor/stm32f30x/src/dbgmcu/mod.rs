#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - MCU Device ID Code Register"]
    pub idcode: IDCODE,
    #[doc = "0x04 - Debug MCU Configuration Register"]
    pub cr: CR,
    #[doc = "0x08 - APB Low Freeze Register"]
    pub apb1fz: APB1FZ,
    #[doc = "0x0c - APB High Freeze Register"]
    pub apb2fz: APB2FZ,
}
#[doc = "MCU Device ID Code Register"]
pub struct IDCODE {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "MCU Device ID Code Register"]
pub mod idcode;
#[doc = "Debug MCU Configuration Register"]
pub struct CR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Debug MCU Configuration Register"]
pub mod cr;
#[doc = "APB Low Freeze Register"]
pub struct APB1FZ {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "APB Low Freeze Register"]
pub mod apb1fz;
#[doc = "APB High Freeze Register"]
pub struct APB2FZ {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "APB High Freeze Register"]
pub mod apb2fz;
