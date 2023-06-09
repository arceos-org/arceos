//! CPUID

use volatile_register::RO;
#[cfg(not(armv6m))]
use volatile_register::RW;

#[cfg(not(armv6m))]
use peripheral::CPUID;

/// Register block
#[repr(C)]
pub struct RegisterBlock {
    /// CPUID base
    pub base: RO<u32>,

    _reserved0: [u32; 15],

    /// Processor Feature (not present on Cortex-M0 variants)
    #[cfg(not(armv6m))]
    pub pfr: [RO<u32>; 2],
    #[cfg(armv6m)]
    _reserved1: [u32; 2],

    /// Debug Feature (not present on Cortex-M0 variants)
    #[cfg(not(armv6m))]
    pub dfr: RO<u32>,
    #[cfg(armv6m)]
    _reserved2: u32,

    /// Auxiliary Feature (not present on Cortex-M0 variants)
    #[cfg(not(armv6m))]
    pub afr: RO<u32>,
    #[cfg(armv6m)]
    _reserved3: u32,

    /// Memory Model Feature (not present on Cortex-M0 variants)
    #[cfg(not(armv6m))]
    pub mmfr: [RO<u32>; 4],
    #[cfg(armv6m)]
    _reserved4: [u32; 4],

    /// Instruction Set Attribute (not present on Cortex-M0 variants)
    #[cfg(not(armv6m))]
    pub isar: [RO<u32>; 5],
    #[cfg(armv6m)]
    _reserved5: [u32; 5],

    _reserved6: u32,

    /// Cache Level ID (only present on Cortex-M7)
    #[cfg(not(armv6m))]
    pub clidr: RO<u32>,

    /// Cache Type (only present on Cortex-M7)
    #[cfg(not(armv6m))]
    pub ctr: RO<u32>,

    /// Cache Size ID (only present on Cortex-M7)
    #[cfg(not(armv6m))]
    pub ccsidr: RO<u32>,

    /// Cache Size Selection (only present on Cortex-M7)
    #[cfg(not(armv6m))]
    pub csselr: RW<u32>,
}

/// Type of cache to select on CSSELR writes.
#[cfg(not(armv6m))]
pub enum CsselrCacheType {
    /// Select DCache or unified cache
    DataOrUnified = 0,
    /// Select ICache
    Instruction = 1,
}

#[cfg(not(armv6m))]
impl CPUID {
    /// Selects the current CCSIDR
    ///
    /// * `level`: the required cache level minus 1, e.g. 0 for L1, 1 for L2
    /// * `ind`: select instruction cache or data/unified cache
    ///
    /// `level` is masked to be between 0 and 7.
    pub fn select_cache(&mut self, level: u8, ind: CsselrCacheType) {
        const CSSELR_IND_POS: u32 = 0;
        const CSSELR_IND_MASK: u32 = 1 << CSSELR_IND_POS;
        const CSSELR_LEVEL_POS: u32 = 1;
        const CSSELR_LEVEL_MASK: u32 = 0x7 << CSSELR_LEVEL_POS;

        unsafe {
            self.csselr.write(
                (((level as u32) << CSSELR_LEVEL_POS) & CSSELR_LEVEL_MASK)
                    | (((ind as u32) << CSSELR_IND_POS) & CSSELR_IND_MASK),
            )
        }
    }

    /// Returns the number of sets and ways in the selected cache
    pub fn cache_num_sets_ways(&mut self, level: u8, ind: CsselrCacheType) -> (u16, u16) {
        const CCSIDR_NUMSETS_POS: u32 = 13;
        const CCSIDR_NUMSETS_MASK: u32 = 0x7FFF << CCSIDR_NUMSETS_POS;
        const CCSIDR_ASSOCIATIVITY_POS: u32 = 3;
        const CCSIDR_ASSOCIATIVITY_MASK: u32 = 0x3FF << CCSIDR_ASSOCIATIVITY_POS;

        self.select_cache(level, ind);
        ::asm::dsb();
        let ccsidr = self.ccsidr.read();
        (
            (1 + ((ccsidr & CCSIDR_NUMSETS_MASK) >> CCSIDR_NUMSETS_POS)) as u16,
            (1 + ((ccsidr & CCSIDR_ASSOCIATIVITY_MASK) >> CCSIDR_ASSOCIATIVITY_POS)) as u16,
        )
    }
}
