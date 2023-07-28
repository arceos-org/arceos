#[doc = r" Value read from the register"]
pub struct R {
    bits: u32,
}
impl super::RI0R {
    #[doc = r" Reads the contents of the register"]
    #[inline]
    pub fn read(&self) -> R {
        R {
            bits: self.register.get(),
        }
    }
}
#[doc = r" Value of the field"]
pub struct STIDR {
    bits: u16,
}
impl STIDR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u16 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct EXIDR {
    bits: u32,
}
impl EXIDR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u32 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct IDER {
    bits: bool,
}
impl IDER {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bit(&self) -> bool {
        self.bits
    }
    #[doc = r" Returns `true` if the bit is clear (0)"]
    #[inline]
    pub fn bit_is_clear(&self) -> bool {
        !self.bit()
    }
    #[doc = r" Returns `true` if the bit is set (1)"]
    #[inline]
    pub fn bit_is_set(&self) -> bool {
        self.bit()
    }
}
#[doc = r" Value of the field"]
pub struct RTRR {
    bits: bool,
}
impl RTRR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bit(&self) -> bool {
        self.bits
    }
    #[doc = r" Returns `true` if the bit is clear (0)"]
    #[inline]
    pub fn bit_is_clear(&self) -> bool {
        !self.bit()
    }
    #[doc = r" Returns `true` if the bit is set (1)"]
    #[inline]
    pub fn bit_is_set(&self) -> bool {
        self.bit()
    }
}
impl R {
    #[doc = r" Value of the register as raw bits"]
    #[inline]
    pub fn bits(&self) -> u32 {
        self.bits
    }
    #[doc = "Bits 21:31 - STID"]
    #[inline]
    pub fn stid(&self) -> STIDR {
        let bits = {
            const MASK: u16 = 2047;
            const OFFSET: u8 = 21;
            ((self.bits >> OFFSET) & MASK as u32) as u16
        };
        STIDR { bits }
    }
    #[doc = "Bits 3:20 - EXID"]
    #[inline]
    pub fn exid(&self) -> EXIDR {
        let bits = {
            const MASK: u32 = 262143;
            const OFFSET: u8 = 3;
            ((self.bits >> OFFSET) & MASK as u32) as u32
        };
        EXIDR { bits }
    }
    #[doc = "Bit 2 - IDE"]
    #[inline]
    pub fn ide(&self) -> IDER {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 2;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        IDER { bits }
    }
    #[doc = "Bit 1 - RTR"]
    #[inline]
    pub fn rtr(&self) -> RTRR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 1;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        RTRR { bits }
    }
}
