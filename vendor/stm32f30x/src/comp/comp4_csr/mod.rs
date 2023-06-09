#[doc = r" Value read from the register"]
pub struct R {
    bits: u32,
}
#[doc = r" Value to write to the register"]
pub struct W {
    bits: u32,
}
impl super::COMP4_CSR {
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
pub struct COMP4ENR {
    bits: bool,
}
impl COMP4ENR {
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
pub struct COMP4MODER {
    bits: u8,
}
impl COMP4MODER {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct COMP4INSELR {
    bits: u8,
}
impl COMP4INSELR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct COMP4INPSELR {
    bits: bool,
}
impl COMP4INPSELR {
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
pub struct COM4WINMODER {
    bits: bool,
}
impl COM4WINMODER {
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
pub struct COMP4_OUT_SELR {
    bits: u8,
}
impl COMP4_OUT_SELR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct COMP4POLR {
    bits: bool,
}
impl COMP4POLR {
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
pub struct COMP4HYSTR {
    bits: u8,
}
impl COMP4HYSTR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct COMP4_BLANKINGR {
    bits: u8,
}
impl COMP4_BLANKINGR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct COMP4OUTR {
    bits: bool,
}
impl COMP4OUTR {
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
pub struct COMP4LOCKR {
    bits: bool,
}
impl COMP4LOCKR {
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
pub struct _COMP4ENW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP4ENW<'a> {
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
pub struct _COMP4MODEW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP4MODEW<'a> {
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
pub struct _COMP4INSELW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP4INSELW<'a> {
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
pub struct _COMP4INPSELW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP4INPSELW<'a> {
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
        const OFFSET: u8 = 7;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _COM4WINMODEW<'a> {
    w: &'a mut W,
}
impl<'a> _COM4WINMODEW<'a> {
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
        const OFFSET: u8 = 9;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _COMP4_OUT_SELW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP4_OUT_SELW<'a> {
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
pub struct _COMP4POLW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP4POLW<'a> {
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
pub struct _COMP4HYSTW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP4HYSTW<'a> {
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
pub struct _COMP4_BLANKINGW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP4_BLANKINGW<'a> {
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
pub struct _COMP4LOCKW<'a> {
    w: &'a mut W,
}
impl<'a> _COMP4LOCKW<'a> {
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
    #[doc = "Bit 0 - Comparator 4 enable"]
    #[inline]
    pub fn comp4en(&self) -> COMP4ENR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 0;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        COMP4ENR { bits }
    }
    #[doc = "Bits 2:3 - Comparator 4 mode"]
    #[inline]
    pub fn comp4mode(&self) -> COMP4MODER {
        let bits = {
            const MASK: u8 = 3;
            const OFFSET: u8 = 2;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        COMP4MODER { bits }
    }
    #[doc = "Bits 4:6 - Comparator 4 inverting input selection"]
    #[inline]
    pub fn comp4insel(&self) -> COMP4INSELR {
        let bits = {
            const MASK: u8 = 7;
            const OFFSET: u8 = 4;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        COMP4INSELR { bits }
    }
    #[doc = "Bit 7 - Comparator 4 non inverted input selection"]
    #[inline]
    pub fn comp4inpsel(&self) -> COMP4INPSELR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 7;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        COMP4INPSELR { bits }
    }
    #[doc = "Bit 9 - Comparator 4 window mode"]
    #[inline]
    pub fn com4winmode(&self) -> COM4WINMODER {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 9;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        COM4WINMODER { bits }
    }
    #[doc = "Bits 10:13 - Comparator 4 output selection"]
    #[inline]
    pub fn comp4_out_sel(&self) -> COMP4_OUT_SELR {
        let bits = {
            const MASK: u8 = 15;
            const OFFSET: u8 = 10;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        COMP4_OUT_SELR { bits }
    }
    #[doc = "Bit 15 - Comparator 4 output polarity"]
    #[inline]
    pub fn comp4pol(&self) -> COMP4POLR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 15;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        COMP4POLR { bits }
    }
    #[doc = "Bits 16:17 - Comparator 4 hysteresis"]
    #[inline]
    pub fn comp4hyst(&self) -> COMP4HYSTR {
        let bits = {
            const MASK: u8 = 3;
            const OFFSET: u8 = 16;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        COMP4HYSTR { bits }
    }
    #[doc = "Bits 18:20 - Comparator 4 blanking source"]
    #[inline]
    pub fn comp4_blanking(&self) -> COMP4_BLANKINGR {
        let bits = {
            const MASK: u8 = 7;
            const OFFSET: u8 = 18;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        COMP4_BLANKINGR { bits }
    }
    #[doc = "Bit 30 - Comparator 4 output"]
    #[inline]
    pub fn comp4out(&self) -> COMP4OUTR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 30;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        COMP4OUTR { bits }
    }
    #[doc = "Bit 31 - Comparator 4 lock"]
    #[inline]
    pub fn comp4lock(&self) -> COMP4LOCKR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 31;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        COMP4LOCKR { bits }
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
    #[doc = "Bit 0 - Comparator 4 enable"]
    #[inline]
    pub fn comp4en(&mut self) -> _COMP4ENW {
        _COMP4ENW { w: self }
    }
    #[doc = "Bits 2:3 - Comparator 4 mode"]
    #[inline]
    pub fn comp4mode(&mut self) -> _COMP4MODEW {
        _COMP4MODEW { w: self }
    }
    #[doc = "Bits 4:6 - Comparator 4 inverting input selection"]
    #[inline]
    pub fn comp4insel(&mut self) -> _COMP4INSELW {
        _COMP4INSELW { w: self }
    }
    #[doc = "Bit 7 - Comparator 4 non inverted input selection"]
    #[inline]
    pub fn comp4inpsel(&mut self) -> _COMP4INPSELW {
        _COMP4INPSELW { w: self }
    }
    #[doc = "Bit 9 - Comparator 4 window mode"]
    #[inline]
    pub fn com4winmode(&mut self) -> _COM4WINMODEW {
        _COM4WINMODEW { w: self }
    }
    #[doc = "Bits 10:13 - Comparator 4 output selection"]
    #[inline]
    pub fn comp4_out_sel(&mut self) -> _COMP4_OUT_SELW {
        _COMP4_OUT_SELW { w: self }
    }
    #[doc = "Bit 15 - Comparator 4 output polarity"]
    #[inline]
    pub fn comp4pol(&mut self) -> _COMP4POLW {
        _COMP4POLW { w: self }
    }
    #[doc = "Bits 16:17 - Comparator 4 hysteresis"]
    #[inline]
    pub fn comp4hyst(&mut self) -> _COMP4HYSTW {
        _COMP4HYSTW { w: self }
    }
    #[doc = "Bits 18:20 - Comparator 4 blanking source"]
    #[inline]
    pub fn comp4_blanking(&mut self) -> _COMP4_BLANKINGW {
        _COMP4_BLANKINGW { w: self }
    }
    #[doc = "Bit 31 - Comparator 4 lock"]
    #[inline]
    pub fn comp4lock(&mut self) -> _COMP4LOCKW {
        _COMP4LOCKW { w: self }
    }
}
