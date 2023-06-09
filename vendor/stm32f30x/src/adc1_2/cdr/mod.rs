#[doc = r" Value read from the register"]
pub struct R {
    bits: u32,
}
impl super::CDR {
    #[doc = r" Reads the contents of the register"]
    #[inline]
    pub fn read(&self) -> R {
        R {
            bits: self.register.get(),
        }
    }
}
#[doc = r" Value of the field"]
pub struct RDATA_SLVR {
    bits: u16,
}
impl RDATA_SLVR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u16 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct RDATA_MSTR {
    bits: u16,
}
impl RDATA_MSTR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u16 {
        self.bits
    }
}
impl R {
    #[doc = r" Value of the register as raw bits"]
    #[inline]
    pub fn bits(&self) -> u32 {
        self.bits
    }
    #[doc = "Bits 16:31 - Regular data of the slave ADC"]
    #[inline]
    pub fn rdata_slv(&self) -> RDATA_SLVR {
        let bits = {
            const MASK: u16 = 65535;
            const OFFSET: u8 = 16;
            ((self.bits >> OFFSET) & MASK as u32) as u16
        };
        RDATA_SLVR { bits }
    }
    #[doc = "Bits 0:15 - Regular data of the master ADC"]
    #[inline]
    pub fn rdata_mst(&self) -> RDATA_MSTR {
        let bits = {
            const MASK: u16 = 65535;
            const OFFSET: u8 = 0;
            ((self.bits >> OFFSET) & MASK as u32) as u16
        };
        RDATA_MSTR { bits }
    }
}
