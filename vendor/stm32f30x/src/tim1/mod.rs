#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - control register 1"]
    pub cr1: CR1,
    #[doc = "0x04 - control register 2"]
    pub cr2: CR2,
    #[doc = "0x08 - slave mode control register"]
    pub smcr: SMCR,
    #[doc = "0x0c - DMA/Interrupt enable register"]
    pub dier: DIER,
    #[doc = "0x10 - status register"]
    pub sr: SR,
    #[doc = "0x14 - event generation register"]
    pub egr: EGR,
    #[doc = "0x18 - capture/compare mode register (output mode)"]
    pub ccmr1_output: CCMR1_OUTPUT,
    #[doc = "0x1c - capture/compare mode register (output mode)"]
    pub ccmr2_output: CCMR2_OUTPUT,
    #[doc = "0x20 - capture/compare enable register"]
    pub ccer: CCER,
    #[doc = "0x24 - counter"]
    pub cnt: CNT,
    #[doc = "0x28 - prescaler"]
    pub psc: PSC,
    #[doc = "0x2c - auto-reload register"]
    pub arr: ARR,
    #[doc = "0x30 - repetition counter register"]
    pub rcr: RCR,
    #[doc = "0x34 - capture/compare register 1"]
    pub ccr1: CCR1,
    #[doc = "0x38 - capture/compare register 2"]
    pub ccr2: CCR2,
    #[doc = "0x3c - capture/compare register 3"]
    pub ccr3: CCR3,
    #[doc = "0x40 - capture/compare register 4"]
    pub ccr4: CCR4,
    #[doc = "0x44 - break and dead-time register"]
    pub bdtr: BDTR,
    #[doc = "0x48 - DMA control register"]
    pub dcr: DCR,
    #[doc = "0x4c - DMA address for full transfer"]
    pub dmar: DMAR,
    _reserved0: [u8; 4usize],
    #[doc = "0x54 - capture/compare mode register 3 (output mode)"]
    pub ccmr3_output: CCMR3_OUTPUT,
    #[doc = "0x58 - capture/compare register 5"]
    pub ccr5: CCR5,
    #[doc = "0x5c - capture/compare register 6"]
    pub ccr6: CCR6,
    #[doc = "0x60 - option registers"]
    pub or: OR,
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
#[doc = "slave mode control register"]
pub struct SMCR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "slave mode control register"]
pub mod smcr;
#[doc = "DMA/Interrupt enable register"]
pub struct DIER {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA/Interrupt enable register"]
pub mod dier;
#[doc = "status register"]
pub struct SR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "status register"]
pub mod sr;
#[doc = "event generation register"]
pub struct EGR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "event generation register"]
pub mod egr;
#[doc = "capture/compare mode register (output mode)"]
pub struct CCMR1_OUTPUT {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "capture/compare mode register (output mode)"]
pub mod ccmr1_output;
#[doc = "capture/compare mode register 1 (input mode)"]
pub struct CCMR1_INPUT {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "capture/compare mode register 1 (input mode)"]
pub mod ccmr1_input;
#[doc = "capture/compare mode register (output mode)"]
pub struct CCMR2_OUTPUT {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "capture/compare mode register (output mode)"]
pub mod ccmr2_output;
#[doc = "capture/compare mode register 2 (input mode)"]
pub struct CCMR2_INPUT {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "capture/compare mode register 2 (input mode)"]
pub mod ccmr2_input;
#[doc = "capture/compare enable register"]
pub struct CCER {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "capture/compare enable register"]
pub mod ccer;
#[doc = "counter"]
pub struct CNT {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "counter"]
pub mod cnt;
#[doc = "prescaler"]
pub struct PSC {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "prescaler"]
pub mod psc;
#[doc = "auto-reload register"]
pub struct ARR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "auto-reload register"]
pub mod arr;
#[doc = "repetition counter register"]
pub struct RCR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "repetition counter register"]
pub mod rcr;
#[doc = "capture/compare register 1"]
pub struct CCR1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "capture/compare register 1"]
pub mod ccr1;
#[doc = "capture/compare register 2"]
pub struct CCR2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "capture/compare register 2"]
pub mod ccr2;
#[doc = "capture/compare register 3"]
pub struct CCR3 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "capture/compare register 3"]
pub mod ccr3;
#[doc = "capture/compare register 4"]
pub struct CCR4 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "capture/compare register 4"]
pub mod ccr4;
#[doc = "break and dead-time register"]
pub struct BDTR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "break and dead-time register"]
pub mod bdtr;
#[doc = "DMA control register"]
pub struct DCR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA control register"]
pub mod dcr;
#[doc = "DMA address for full transfer"]
pub struct DMAR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "DMA address for full transfer"]
pub mod dmar;
#[doc = "capture/compare mode register 3 (output mode)"]
pub struct CCMR3_OUTPUT {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "capture/compare mode register 3 (output mode)"]
pub mod ccmr3_output;
#[doc = "capture/compare register 5"]
pub struct CCR5 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "capture/compare register 5"]
pub mod ccr5;
#[doc = "capture/compare register 6"]
pub struct CCR6 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "capture/compare register 6"]
pub mod ccr6;
#[doc = "option registers"]
pub struct OR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "option registers"]
pub mod or;
