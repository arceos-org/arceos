#[doc = r" Value read from the register"]
pub struct R {
    bits: u32,
}
impl super::IDCODE {
    #[doc = r" Reads the contents of the register"]
    #[inline]
    pub fn read(&self) -> R {
        R {
            bits: self.register.get(),
        }
    }
}
#[doc = r" Value of the field"]
pub struct DEV_IDR {
    bits: u16,
}
impl DEV_IDR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u16 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct REV_IDR {
    bits: u16,
}
impl REV_IDR {
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
    #[doc = "Bits 0:11 - Device Identifier"]
    #[inline]
    pub fn dev_id(&self) -> DEV_IDR {
        let bits = {
            const MASK: u16 = 4095;
            const OFFSET: u8 = 0;
            ((self.bits >> OFFSET) & MASK as u32) as u16
        };
        DEV_IDR { bits }
    }
    #[doc = "Bits 16:31 - Revision Identifier"]
    #[inline]
    pub fn rev_id(&self) -> REV_IDR {
        let bits = {
            const MASK: u16 = 65535;
            const OFFSET: u8 = 16;
            ((self.bits >> OFFSET) & MASK as u32) as u16
        };
        REV_IDR { bits }
    }
}
