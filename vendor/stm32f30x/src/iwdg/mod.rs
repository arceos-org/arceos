#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Key register"]
    pub kr: KR,
    #[doc = "0x04 - Prescaler register"]
    pub pr: PR,
    #[doc = "0x08 - Reload register"]
    pub rlr: RLR,
    #[doc = "0x0c - Status register"]
    pub sr: SR,
    #[doc = "0x10 - Window register"]
    pub winr: WINR,
}
#[doc = "Key register"]
pub struct KR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Key register"]
pub mod kr;
#[doc = "Prescaler register"]
pub struct PR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Prescaler register"]
pub mod pr;
#[doc = "Reload register"]
pub struct RLR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Reload register"]
pub mod rlr;
#[doc = "Status register"]
pub struct SR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Status register"]
pub mod sr;
#[doc = "Window register"]
pub struct WINR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Window register"]
pub mod winr;
