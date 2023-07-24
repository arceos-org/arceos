#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Flash access control register"]
    pub acr: ACR,
    #[doc = "0x04 - Flash key register"]
    pub keyr: KEYR,
    #[doc = "0x08 - Flash option key register"]
    pub optkeyr: OPTKEYR,
    #[doc = "0x0c - Flash status register"]
    pub sr: SR,
    #[doc = "0x10 - Flash control register"]
    pub cr: CR,
    #[doc = "0x14 - Flash address register"]
    pub ar: AR,
    _reserved0: [u8; 4usize],
    #[doc = "0x1c - Option byte register"]
    pub obr: OBR,
    #[doc = "0x20 - Write protection register"]
    pub wrpr: WRPR,
}
#[doc = "Flash access control register"]
pub struct ACR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Flash access control register"]
pub mod acr;
#[doc = "Flash key register"]
pub struct KEYR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Flash key register"]
pub mod keyr;
#[doc = "Flash option key register"]
pub struct OPTKEYR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Flash option key register"]
pub mod optkeyr;
#[doc = "Flash status register"]
pub struct SR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Flash status register"]
pub mod sr;
#[doc = "Flash control register"]
pub struct CR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Flash control register"]
pub mod cr;
#[doc = "Flash address register"]
pub struct AR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Flash address register"]
pub mod ar;
#[doc = "Option byte register"]
pub struct OBR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Option byte register"]
pub mod obr;
#[doc = "Write protection register"]
pub struct WRPR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Write protection register"]
pub mod wrpr;
