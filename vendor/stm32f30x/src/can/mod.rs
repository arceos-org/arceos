#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - master control register"]
    pub mcr: MCR,
    #[doc = "0x04 - master status register"]
    pub msr: MSR,
    #[doc = "0x08 - transmit status register"]
    pub tsr: TSR,
    #[doc = "0x0c - receive FIFO 0 register"]
    pub rf0r: RF0R,
    #[doc = "0x10 - receive FIFO 1 register"]
    pub rf1r: RF1R,
    #[doc = "0x14 - interrupt enable register"]
    pub ier: IER,
    #[doc = "0x18 - error status register"]
    pub esr: ESR,
    #[doc = "0x1c - bit timing register"]
    pub btr: BTR,
    _reserved0: [u8; 352usize],
    #[doc = "0x180 - TX mailbox identifier register"]
    pub ti0r: TI0R,
    #[doc = "0x184 - mailbox data length control and time stamp register"]
    pub tdt0r: TDT0R,
    #[doc = "0x188 - mailbox data low register"]
    pub tdl0r: TDL0R,
    #[doc = "0x18c - mailbox data high register"]
    pub tdh0r: TDH0R,
    #[doc = "0x190 - TX mailbox identifier register"]
    pub ti1r: TI1R,
    #[doc = "0x194 - mailbox data length control and time stamp register"]
    pub tdt1r: TDT1R,
    #[doc = "0x198 - mailbox data low register"]
    pub tdl1r: TDL1R,
    #[doc = "0x19c - mailbox data high register"]
    pub tdh1r: TDH1R,
    #[doc = "0x1a0 - TX mailbox identifier register"]
    pub ti2r: TI2R,
    #[doc = "0x1a4 - mailbox data length control and time stamp register"]
    pub tdt2r: TDT2R,
    #[doc = "0x1a8 - mailbox data low register"]
    pub tdl2r: TDL2R,
    #[doc = "0x1ac - mailbox data high register"]
    pub tdh2r: TDH2R,
    #[doc = "0x1b0 - receive FIFO mailbox identifier register"]
    pub ri0r: RI0R,
    #[doc = "0x1b4 - receive FIFO mailbox data length control and time stamp register"]
    pub rdt0r: RDT0R,
    #[doc = "0x1b8 - receive FIFO mailbox data low register"]
    pub rdl0r: RDL0R,
    #[doc = "0x1bc - receive FIFO mailbox data high register"]
    pub rdh0r: RDH0R,
    #[doc = "0x1c0 - receive FIFO mailbox identifier register"]
    pub ri1r: RI1R,
    #[doc = "0x1c4 - receive FIFO mailbox data length control and time stamp register"]
    pub rdt1r: RDT1R,
    #[doc = "0x1c8 - receive FIFO mailbox data low register"]
    pub rdl1r: RDL1R,
    #[doc = "0x1cc - receive FIFO mailbox data high register"]
    pub rdh1r: RDH1R,
    _reserved1: [u8; 48usize],
    #[doc = "0x200 - filter master register"]
    pub fmr: FMR,
    #[doc = "0x204 - filter mode register"]
    pub fm1r: FM1R,
    _reserved2: [u8; 4usize],
    #[doc = "0x20c - filter scale register"]
    pub fs1r: FS1R,
    _reserved3: [u8; 4usize],
    #[doc = "0x214 - filter FIFO assignment register"]
    pub ffa1r: FFA1R,
    _reserved4: [u8; 4usize],
    #[doc = "0x21c - CAN filter activation register"]
    pub fa1r: FA1R,
    _reserved5: [u8; 32usize],
    #[doc = "0x240 - Filter bank 0 register 1"]
    pub f0r1: F0R1,
    #[doc = "0x244 - Filter bank 0 register 2"]
    pub f0r2: F0R2,
    #[doc = "0x248 - Filter bank 1 register 1"]
    pub f1r1: F1R1,
    #[doc = "0x24c - Filter bank 1 register 2"]
    pub f1r2: F1R2,
    #[doc = "0x250 - Filter bank 2 register 1"]
    pub f2r1: F2R1,
    #[doc = "0x254 - Filter bank 2 register 2"]
    pub f2r2: F2R2,
    #[doc = "0x258 - Filter bank 3 register 1"]
    pub f3r1: F3R1,
    #[doc = "0x25c - Filter bank 3 register 2"]
    pub f3r2: F3R2,
    #[doc = "0x260 - Filter bank 4 register 1"]
    pub f4r1: F4R1,
    #[doc = "0x264 - Filter bank 4 register 2"]
    pub f4r2: F4R2,
    #[doc = "0x268 - Filter bank 5 register 1"]
    pub f5r1: F5R1,
    #[doc = "0x26c - Filter bank 5 register 2"]
    pub f5r2: F5R2,
    #[doc = "0x270 - Filter bank 6 register 1"]
    pub f6r1: F6R1,
    #[doc = "0x274 - Filter bank 6 register 2"]
    pub f6r2: F6R2,
    #[doc = "0x278 - Filter bank 7 register 1"]
    pub f7r1: F7R1,
    #[doc = "0x27c - Filter bank 7 register 2"]
    pub f7r2: F7R2,
    #[doc = "0x280 - Filter bank 8 register 1"]
    pub f8r1: F8R1,
    #[doc = "0x284 - Filter bank 8 register 2"]
    pub f8r2: F8R2,
    #[doc = "0x288 - Filter bank 9 register 1"]
    pub f9r1: F9R1,
    #[doc = "0x28c - Filter bank 9 register 2"]
    pub f9r2: F9R2,
    #[doc = "0x290 - Filter bank 10 register 1"]
    pub f10r1: F10R1,
    #[doc = "0x294 - Filter bank 10 register 2"]
    pub f10r2: F10R2,
    #[doc = "0x298 - Filter bank 11 register 1"]
    pub f11r1: F11R1,
    #[doc = "0x29c - Filter bank 11 register 2"]
    pub f11r2: F11R2,
    #[doc = "0x2a0 - Filter bank 4 register 1"]
    pub f12r1: F12R1,
    #[doc = "0x2a4 - Filter bank 12 register 2"]
    pub f12r2: F12R2,
    #[doc = "0x2a8 - Filter bank 13 register 1"]
    pub f13r1: F13R1,
    #[doc = "0x2ac - Filter bank 13 register 2"]
    pub f13r2: F13R2,
    #[doc = "0x2b0 - Filter bank 14 register 1"]
    pub f14r1: F14R1,
    #[doc = "0x2b4 - Filter bank 14 register 2"]
    pub f14r2: F14R2,
    #[doc = "0x2b8 - Filter bank 15 register 1"]
    pub f15r1: F15R1,
    #[doc = "0x2bc - Filter bank 15 register 2"]
    pub f15r2: F15R2,
    #[doc = "0x2c0 - Filter bank 16 register 1"]
    pub f16r1: F16R1,
    #[doc = "0x2c4 - Filter bank 16 register 2"]
    pub f16r2: F16R2,
    #[doc = "0x2c8 - Filter bank 17 register 1"]
    pub f17r1: F17R1,
    #[doc = "0x2cc - Filter bank 17 register 2"]
    pub f17r2: F17R2,
    #[doc = "0x2d0 - Filter bank 18 register 1"]
    pub f18r1: F18R1,
    #[doc = "0x2d4 - Filter bank 18 register 2"]
    pub f18r2: F18R2,
    #[doc = "0x2d8 - Filter bank 19 register 1"]
    pub f19r1: F19R1,
    #[doc = "0x2dc - Filter bank 19 register 2"]
    pub f19r2: F19R2,
    #[doc = "0x2e0 - Filter bank 20 register 1"]
    pub f20r1: F20R1,
    #[doc = "0x2e4 - Filter bank 20 register 2"]
    pub f20r2: F20R2,
    #[doc = "0x2e8 - Filter bank 21 register 1"]
    pub f21r1: F21R1,
    #[doc = "0x2ec - Filter bank 21 register 2"]
    pub f21r2: F21R2,
    #[doc = "0x2f0 - Filter bank 22 register 1"]
    pub f22r1: F22R1,
    #[doc = "0x2f4 - Filter bank 22 register 2"]
    pub f22r2: F22R2,
    #[doc = "0x2f8 - Filter bank 23 register 1"]
    pub f23r1: F23R1,
    #[doc = "0x2fc - Filter bank 23 register 2"]
    pub f23r2: F23R2,
    #[doc = "0x300 - Filter bank 24 register 1"]
    pub f24r1: F24R1,
    #[doc = "0x304 - Filter bank 24 register 2"]
    pub f24r2: F24R2,
    #[doc = "0x308 - Filter bank 25 register 1"]
    pub f25r1: F25R1,
    #[doc = "0x30c - Filter bank 25 register 2"]
    pub f25r2: F25R2,
    #[doc = "0x310 - Filter bank 26 register 1"]
    pub f26r1: F26R1,
    #[doc = "0x314 - Filter bank 26 register 2"]
    pub f26r2: F26R2,
    #[doc = "0x318 - Filter bank 27 register 1"]
    pub f27r1: F27R1,
    #[doc = "0x31c - Filter bank 27 register 2"]
    pub f27r2: F27R2,
}
#[doc = "master control register"]
pub struct MCR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "master control register"]
pub mod mcr;
#[doc = "master status register"]
pub struct MSR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "master status register"]
pub mod msr;
#[doc = "transmit status register"]
pub struct TSR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "transmit status register"]
pub mod tsr;
#[doc = "receive FIFO 0 register"]
pub struct RF0R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "receive FIFO 0 register"]
pub mod rf0r;
#[doc = "receive FIFO 1 register"]
pub struct RF1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "receive FIFO 1 register"]
pub mod rf1r;
#[doc = "interrupt enable register"]
pub struct IER {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "interrupt enable register"]
pub mod ier;
#[doc = "error status register"]
pub struct ESR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "error status register"]
pub mod esr;
#[doc = "bit timing register"]
pub struct BTR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "bit timing register"]
pub mod btr;
#[doc = "TX mailbox identifier register"]
pub struct TI0R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "TX mailbox identifier register"]
pub mod ti0r;
#[doc = "mailbox data length control and time stamp register"]
pub struct TDT0R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "mailbox data length control and time stamp register"]
pub mod tdt0r;
#[doc = "mailbox data low register"]
pub struct TDL0R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "mailbox data low register"]
pub mod tdl0r;
#[doc = "mailbox data high register"]
pub struct TDH0R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "mailbox data high register"]
pub mod tdh0r;
#[doc = "TX mailbox identifier register"]
pub struct TI1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "TX mailbox identifier register"]
pub mod ti1r;
#[doc = "mailbox data length control and time stamp register"]
pub struct TDT1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "mailbox data length control and time stamp register"]
pub mod tdt1r;
#[doc = "mailbox data low register"]
pub struct TDL1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "mailbox data low register"]
pub mod tdl1r;
#[doc = "mailbox data high register"]
pub struct TDH1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "mailbox data high register"]
pub mod tdh1r;
#[doc = "TX mailbox identifier register"]
pub struct TI2R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "TX mailbox identifier register"]
pub mod ti2r;
#[doc = "mailbox data length control and time stamp register"]
pub struct TDT2R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "mailbox data length control and time stamp register"]
pub mod tdt2r;
#[doc = "mailbox data low register"]
pub struct TDL2R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "mailbox data low register"]
pub mod tdl2r;
#[doc = "mailbox data high register"]
pub struct TDH2R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "mailbox data high register"]
pub mod tdh2r;
#[doc = "receive FIFO mailbox identifier register"]
pub struct RI0R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "receive FIFO mailbox identifier register"]
pub mod ri0r;
#[doc = "receive FIFO mailbox data length control and time stamp register"]
pub struct RDT0R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "receive FIFO mailbox data length control and time stamp register"]
pub mod rdt0r;
#[doc = "receive FIFO mailbox data low register"]
pub struct RDL0R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "receive FIFO mailbox data low register"]
pub mod rdl0r;
#[doc = "receive FIFO mailbox data high register"]
pub struct RDH0R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "receive FIFO mailbox data high register"]
pub mod rdh0r;
#[doc = "receive FIFO mailbox identifier register"]
pub struct RI1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "receive FIFO mailbox identifier register"]
pub mod ri1r;
#[doc = "receive FIFO mailbox data length control and time stamp register"]
pub struct RDT1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "receive FIFO mailbox data length control and time stamp register"]
pub mod rdt1r;
#[doc = "receive FIFO mailbox data low register"]
pub struct RDL1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "receive FIFO mailbox data low register"]
pub mod rdl1r;
#[doc = "receive FIFO mailbox data high register"]
pub struct RDH1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "receive FIFO mailbox data high register"]
pub mod rdh1r;
#[doc = "filter master register"]
pub struct FMR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "filter master register"]
pub mod fmr;
#[doc = "filter mode register"]
pub struct FM1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "filter mode register"]
pub mod fm1r;
#[doc = "filter scale register"]
pub struct FS1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "filter scale register"]
pub mod fs1r;
#[doc = "filter FIFO assignment register"]
pub struct FFA1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "filter FIFO assignment register"]
pub mod ffa1r;
#[doc = "CAN filter activation register"]
pub struct FA1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "CAN filter activation register"]
pub mod fa1r;
#[doc = "Filter bank 0 register 1"]
pub struct F0R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 0 register 1"]
pub mod f0r1;
#[doc = "Filter bank 0 register 2"]
pub struct F0R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 0 register 2"]
pub mod f0r2;
#[doc = "Filter bank 1 register 1"]
pub struct F1R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 1 register 1"]
pub mod f1r1;
#[doc = "Filter bank 1 register 2"]
pub struct F1R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 1 register 2"]
pub mod f1r2;
#[doc = "Filter bank 2 register 1"]
pub struct F2R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 2 register 1"]
pub mod f2r1;
#[doc = "Filter bank 2 register 2"]
pub struct F2R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 2 register 2"]
pub mod f2r2;
#[doc = "Filter bank 3 register 1"]
pub struct F3R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 3 register 1"]
pub mod f3r1;
#[doc = "Filter bank 3 register 2"]
pub struct F3R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 3 register 2"]
pub mod f3r2;
#[doc = "Filter bank 4 register 1"]
pub struct F4R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 4 register 1"]
pub mod f4r1;
#[doc = "Filter bank 4 register 2"]
pub struct F4R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 4 register 2"]
pub mod f4r2;
#[doc = "Filter bank 5 register 1"]
pub struct F5R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 5 register 1"]
pub mod f5r1;
#[doc = "Filter bank 5 register 2"]
pub struct F5R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 5 register 2"]
pub mod f5r2;
#[doc = "Filter bank 6 register 1"]
pub struct F6R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 6 register 1"]
pub mod f6r1;
#[doc = "Filter bank 6 register 2"]
pub struct F6R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 6 register 2"]
pub mod f6r2;
#[doc = "Filter bank 7 register 1"]
pub struct F7R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 7 register 1"]
pub mod f7r1;
#[doc = "Filter bank 7 register 2"]
pub struct F7R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 7 register 2"]
pub mod f7r2;
#[doc = "Filter bank 8 register 1"]
pub struct F8R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 8 register 1"]
pub mod f8r1;
#[doc = "Filter bank 8 register 2"]
pub struct F8R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 8 register 2"]
pub mod f8r2;
#[doc = "Filter bank 9 register 1"]
pub struct F9R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 9 register 1"]
pub mod f9r1;
#[doc = "Filter bank 9 register 2"]
pub struct F9R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 9 register 2"]
pub mod f9r2;
#[doc = "Filter bank 10 register 1"]
pub struct F10R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 10 register 1"]
pub mod f10r1;
#[doc = "Filter bank 10 register 2"]
pub struct F10R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 10 register 2"]
pub mod f10r2;
#[doc = "Filter bank 11 register 1"]
pub struct F11R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 11 register 1"]
pub mod f11r1;
#[doc = "Filter bank 11 register 2"]
pub struct F11R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 11 register 2"]
pub mod f11r2;
#[doc = "Filter bank 4 register 1"]
pub struct F12R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 4 register 1"]
pub mod f12r1;
#[doc = "Filter bank 12 register 2"]
pub struct F12R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 12 register 2"]
pub mod f12r2;
#[doc = "Filter bank 13 register 1"]
pub struct F13R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 13 register 1"]
pub mod f13r1;
#[doc = "Filter bank 13 register 2"]
pub struct F13R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 13 register 2"]
pub mod f13r2;
#[doc = "Filter bank 14 register 1"]
pub struct F14R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 14 register 1"]
pub mod f14r1;
#[doc = "Filter bank 14 register 2"]
pub struct F14R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 14 register 2"]
pub mod f14r2;
#[doc = "Filter bank 15 register 1"]
pub struct F15R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 15 register 1"]
pub mod f15r1;
#[doc = "Filter bank 15 register 2"]
pub struct F15R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 15 register 2"]
pub mod f15r2;
#[doc = "Filter bank 16 register 1"]
pub struct F16R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 16 register 1"]
pub mod f16r1;
#[doc = "Filter bank 16 register 2"]
pub struct F16R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 16 register 2"]
pub mod f16r2;
#[doc = "Filter bank 17 register 1"]
pub struct F17R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 17 register 1"]
pub mod f17r1;
#[doc = "Filter bank 17 register 2"]
pub struct F17R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 17 register 2"]
pub mod f17r2;
#[doc = "Filter bank 18 register 1"]
pub struct F18R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 18 register 1"]
pub mod f18r1;
#[doc = "Filter bank 18 register 2"]
pub struct F18R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 18 register 2"]
pub mod f18r2;
#[doc = "Filter bank 19 register 1"]
pub struct F19R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 19 register 1"]
pub mod f19r1;
#[doc = "Filter bank 19 register 2"]
pub struct F19R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 19 register 2"]
pub mod f19r2;
#[doc = "Filter bank 20 register 1"]
pub struct F20R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 20 register 1"]
pub mod f20r1;
#[doc = "Filter bank 20 register 2"]
pub struct F20R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 20 register 2"]
pub mod f20r2;
#[doc = "Filter bank 21 register 1"]
pub struct F21R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 21 register 1"]
pub mod f21r1;
#[doc = "Filter bank 21 register 2"]
pub struct F21R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 21 register 2"]
pub mod f21r2;
#[doc = "Filter bank 22 register 1"]
pub struct F22R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 22 register 1"]
pub mod f22r1;
#[doc = "Filter bank 22 register 2"]
pub struct F22R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 22 register 2"]
pub mod f22r2;
#[doc = "Filter bank 23 register 1"]
pub struct F23R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 23 register 1"]
pub mod f23r1;
#[doc = "Filter bank 23 register 2"]
pub struct F23R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 23 register 2"]
pub mod f23r2;
#[doc = "Filter bank 24 register 1"]
pub struct F24R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 24 register 1"]
pub mod f24r1;
#[doc = "Filter bank 24 register 2"]
pub struct F24R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 24 register 2"]
pub mod f24r2;
#[doc = "Filter bank 25 register 1"]
pub struct F25R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 25 register 1"]
pub mod f25r1;
#[doc = "Filter bank 25 register 2"]
pub struct F25R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 25 register 2"]
pub mod f25r2;
#[doc = "Filter bank 26 register 1"]
pub struct F26R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 26 register 1"]
pub mod f26r1;
#[doc = "Filter bank 26 register 2"]
pub struct F26R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 26 register 2"]
pub mod f26r2;
#[doc = "Filter bank 27 register 1"]
pub struct F27R1 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 27 register 1"]
pub mod f27r1;
#[doc = "Filter bank 27 register 2"]
pub struct F27R2 {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Filter bank 27 register 2"]
pub mod f27r2;
