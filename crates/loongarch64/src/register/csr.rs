pub trait Register {
    fn read() -> Self;
    fn write(&mut self);
}
pub const CSR_CRMD: usize = 0x0;
pub const CSR_PRMD: usize = 0x1;
pub const CSR_EUEN: usize = 0x2;
pub const CSR_MISC: usize = 0x3;
pub const CSR_ECFG: usize = 0x4;
pub const CSR_ESTAT: usize = 0x5;
pub const CSR_ERA: usize = 0x6;
pub const CSR_BADV: usize = 0x7;
pub const CSR_BADI: usize = 0x8;
pub const CSR_EENTRY: usize = 0xC;
pub const CSR_TLBIDX: usize = 0x10;
pub const CSR_TLBEHI: usize = 0x11;

pub const CSR_TLBELO: usize = 0x12;

pub const CSR_ASID: usize = 0x18;
pub const CSR_PGDL: usize = 0x19;
pub const CSR_PGDH: usize = 0x1A;
pub const CSR_PGD: usize = 0x1B;
pub const CSR_PWCL: usize = 0x1C;
pub const CSR_PWCH: usize = 0x1D;
pub const CSR_STLBPS: usize = 0x1E;
pub const CSR_RVACFG: usize = 0x1F;
pub const CSR_CPUID: usize = 0x20;
pub const CSR_PRCFG1: usize = 0x21;
pub const CSR_PRCFG2: usize = 0x22;
pub const CSR_PRCFG3: usize = 0x23;
pub const CSR_SAVE: usize = 0x30; //0x30 + n(n[0-15]
pub const CSR_TID: usize = 0x40;
pub const CSR_TCFG: usize = 0x41;
pub const CSR_TVAL: usize = 0x42;
pub const CSR_CNTC: usize = 0x43;
pub const CSR_TICLR: usize = 0x44;

pub const CSR_TLBRENTRY: usize = 0x88;
pub const CSR_TLBRBADV: usize = 0x89;
pub const CSR_TLBRERA: usize = 0x8A;
pub const CSR_TLBRSAVE: usize = 0x8B;

pub const CSR_TLBRELO: usize = 0x8C;

pub const CSR_TLBREHI: usize = 0x8E;
pub const CSR_TLBRPRMD: usize = 0x8F;

pub const CSR_MERRCTL: usize = 0x90;
pub const CSR_MERRINF01: usize = 0x91;
pub const CSR_MERRINF02: usize = 0x92;
pub const CSR_MERRENTRY: usize = 0x93;
pub const CSR_MEMRERA: usize = 0x94;
pub const CSR_MEMRSAVE: usize = 0x95;

pub const CSR_CTAG: usize = 0x98;

pub const CSR_DMW: usize = 0x180; //0x180 + n(n[0-3]
