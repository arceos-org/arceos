#[doc = r" Value read from the register"]
pub struct R {
    bits: u32,
}
#[doc = r" Value to write to the register"]
pub struct W {
    bits: u32,
}
impl super::COMP1_CSR {
    #[doc = r" Modifies the contents of the register"]
    #[inline]
    pub fn modify<F>(&self, f: F)
    where
        for<'w> F: FnOnce(&R, &'w mut W) -> &'w mut W,
    {
        let bits = self.register.get();
        let r = R { bits: bits };
        let mut w = W { bits: bits };
        f(&r, &mut w);
        self.register.set(w.bits);
    }
    #[doc = r" Reads the contents of the register"]
    #[inline]
    pub fn read(&self) -> R {
        R {
            bits: self.register.get(),
        }
    }
    #[doc = r" Writes to the register"]
    #[inline]
    pub fn write<F>(&self, f: F)
    where
        F: FnOnce(&mut W) -> &mut W,
    {
        let mut w = W::reset_value();
        f(&mut w);
        self.register.set(w.bits);
    }
    #[doc = r" Writes the reset value to the register"]
    #[inline]
    pub fn reset(&self) {
        self.write(|w| w)
    }
}
#[doc = r" Value of the field"]
pub struct COMP1ENR {
    bits: bool,
}
impl COMP1ENR {
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
pub struct COMP1_INP_DACR {
    bits: bool,
}
impl COMP1_INP_DACR {
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
pub struct COMP1MODER {
    bits: u8,
}
impl COMP1MODER {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct COMP1INSELR {
    bits: u8,
}
impl COMP1INSELR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct COMP1_OUT_SELR {
    bits: u8,
}
impl COMP1_OUT_SELR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct COMP1POLR {
    bits: bool,
}
impl COMP1POLR {
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
pub struct COMP1HYSTR {
    bits: u8,
}
impl COMP1HYSTR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct COMP1_BLANKINGR {
    bits: u8,
}
impl COMP1_BLANKINGR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct COMP1OUTR {
    bits: bool,
}
impl COMP1OUTR {
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
pub struct COMP1LOCKR {
    bits: bool,
}
impl COMP1LOCKR {
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
#[doc = r" Proxy"]
pub struct _COMP1ENW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP1ENW<'a> {
    #[doc = r" Sets the field bit"]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r" Clears the field bit"]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bit(self, value: bool) -> &'a mut W {
        const MASK: bool = true;
        const OFFSET: u8 = 0;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _COMP1_INP_DACW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP1_INP_DACW<'a> {
    #[doc = r" Sets the field bit"]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r" Clears the field bit"]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bit(self, value: bool) -> &'a mut W {
        const MASK: bool = true;
        const OFFSET: u8 = 1;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _COMP1MODEW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP1MODEW<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 2;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _COMP1INSELW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP1INSELW<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 7;
        const OFFSET: u8 = 4;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _COMP1_OUT_SELW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP1_OUT_SELW<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 15;
        const OFFSET: u8 = 10;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _COMP1POLW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP1POLW<'a> {
    #[doc = r" Sets the field bit"]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r" Clears the field bit"]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bit(self, value: bool) -> &'a mut W {
        const MASK: bool = true;
        const OFFSET: u8 = 15;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _COMP1HYSTW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP1HYSTW<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 16;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _COMP1_BLANKINGW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP1_BLANKINGW<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 7;
        const OFFSET: u8 = 18;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _COMP1LOCKW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP1LOCKW<'a> {
    #[doc = r" Sets the field bit"]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r" Clears the field bit"]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bit(self, value: bool) -> &'a mut W {
        const MASK: bool = true;
        const OFFSET: u8 = 31;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
impl R {
    #[doc = r" Value of the register as raw bits"]
    #[inline]
    pub fn bits(&self) -> u32 {
        self.bits
    }
    #[doc = "Bit 0 - Comparator 1 enable"]
    #[inline]
    pub fn comp1en(&self) -> COMP1ENR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 0;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        COMP1ENR { bits }
    }
    #[doc = "Bit 1 - COMP1_INP_DAC"]
    #[inline]
    pub fn comp1_inp_dac(&self) -> COMP1_INP_DACR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 1;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        COMP1_INP_DACR { bits }
    }
    #[doc = "Bits 2:3 - Comparator 1 mode"]
    #[inline]
    pub fn comp1mode(&self) -> COMP1MODER {
        let bits = {
            const MASK: u8 = 3;
            const OFFSET: u8 = 2;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        COMP1MODER { bits }
    }
    #[doc = "Bits 4:6 - Comparator 1 inverting input selection"]
    #[inline]
    pub fn comp1insel(&self) -> COMP1INSELR {
        let bits = {
            const MASK: u8 = 7;
            const OFFSET: u8 = 4;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        COMP1INSELR { bits }
    }
    #[doc = "Bits 10:13 - Comparator 1 output selection"]
    #[inline]
    pub fn comp1_out_sel(&self) -> COMP1_OUT_SELR {
        let bits = {
            const MASK: u8 = 15;
            const OFFSET: u8 = 10;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        COMP1_OUT_SELR { bits }
    }
    #[doc = "Bit 15 - Comparator 1 output polarity"]
    #[inline]
    pub fn comp1pol(&self) -> COMP1POLR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 15;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        COMP1POLR { bits }
    }
    #[doc = "Bits 16:17 - Comparator 1 hysteresis"]
    #[inline]
    pub fn comp1hyst(&self) -> COMP1HYSTR {
        let bits = {
            const MASK: u8 = 3;
            const OFFSET: u8 = 16;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        COMP1HYSTR { bits }
    }
    #[doc = "Bits 18:20 - Comparator 1 blanking source"]
    #[inline]
    pub fn comp1_blanking(&self) -> COMP1_BLANKINGR {
        let bits = {
            const MASK: u8 = 7;
            const OFFSET: u8 = 18;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        COMP1_BLANKINGR { bits }
    }
    #[doc = "Bit 30 - Comparator 1 output"]
    #[inline]
    pub fn comp1out(&self) -> COMP1OUTR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 30;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        COMP1OUTR { bits }
    }
    #[doc = "Bit 31 - Comparator 1 lock"]
    #[inline]
    pub fn comp1lock(&self) -> COMP1LOCKR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 31;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        COMP1LOCKR { bits }
    }
}
impl W {
    #[doc = r" Reset value of the register"]
    #[inline]
    pub fn reset_value() -> W {
        W { bits: 0 }
    }
    #[doc = r" Writes raw bits to the register"]
    #[inline]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.bits = bits;
        self
    }
    #[doc = "Bit 0 - Comparator 1 enable"]
    #[inline]
    pub fn comp1en(&mut self) -> _COMP1ENW {
        _COMP1ENW { w: self }
    }
    #[doc = "Bit 1 - COMP1_INP_DAC"]
    #[inline]
    pub fn comp1_inp_dac(&mut self) -> _COMP1_INP_DACW {
        _COMP1_INP_DACW { w: self }
    }
    #[doc = "Bits 2:3 - Comparator 1 mode"]
    #[inline]
    pub fn comp1mode(&mut self) -> _COMP1MODEW {
        _COMP1MODEW { w: self }
    }
    #[doc = "Bits 4:6 - Comparator 1 inverting input selection"]
    #[inline]
    pub fn comp1insel(&mut self) -> _COMP1INSELW {
        _COMP1INSELW { w: self }
    }
    #[doc = "Bits 10:13 - Comparator 1 output selection"]
    #[inline]
    pub fn comp1_out_sel(&mut self) -> _COMP1_OUT_SELW {
        _COMP1_OUT_SELW { w: self }
    }
    #[doc = "Bit 15 - Comparator 1 output polarity"]
    #[inline]
    pub fn comp1pol(&mut self) -> _COMP1POLW {
        _COMP1POLW { w: self }
    }
    #[doc = "Bits 16:17 - Comparator 1 hysteresis"]
    #[inline]
    pub fn comp1hyst(&mut self) -> _COMP1HYSTW {
        _COMP1HYSTW { w: self }
    }
    #[doc = "Bits 18:20 - Comparator 1 blanking source"]
    #[inline]
    pub fn comp1_blanking(&mut self) -> _COMP1_BLANKINGW {
        _COMP1_BLANKINGW { w: self }
    }
    #[doc = "Bit 31 - Comparator 1 lock"]
    #[inline]
    pub fn comp1lock(&mut self) -> _COMP1LOCKW {
        _COMP1LOCKW { w: self }
    }
}
