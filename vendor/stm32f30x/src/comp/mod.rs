#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - control and status register"]
    pub comp1_csr: COMP1_CSR,
    #[doc = "0x04 - control and status register"]
    pub comp2_csr: COMP2_CSR,
    #[doc = "0x08 - control and status register"]
    pub comp3_csr: COMP3_CSR,
    #[doc = "0x0c - control and status register"]
    pub comp4_csr: COMP4_CSR,
    #[doc = "0x10 - control and status register"]
    pub comp5_csr: COMP5_CSR,
    #[doc = "0x14 - control and status register"]
    pub comp6_csr: COMP6_CSR,
    #[doc = "0x18 - control and status register"]
    pub comp7_csr: COMP7_CSR,
}
#[doc = "control and status register"]
pub struct COMP1_CSR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "control and status register"]
pub mod comp1_csr;
#[doc = "control and status register"]
pub struct COMP2_CSR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "control and status register"]
pub mod comp2_csr;
#[doc = "control and status register"]
pub struct COMP3_CSR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "control and status register"]
pub mod comp3_csr;
#[doc = "control and status register"]
pub struct COMP4_CSR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "control and status register"]
pub mod comp4_csr;
#[doc = "control and status register"]
pub struct COMP5_CSR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "control and status register"]
pub mod comp5_csr;
#[doc = "control and status register"]
pub struct COMP6_CSR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "control and status register"]
pub mod comp6_csr;
#[doc = "control and status register"]
pub struct COMP7_CSR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "control and status register"]
pub mod comp7_csr;
