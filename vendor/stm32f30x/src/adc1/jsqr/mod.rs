#[doc = r" Value read from the register"]
pub struct R {
    bits: u32,
}
#[doc = r" Value to write to the register"]
pub struct W {
    bits: u32,
}
impl super::JSQR {
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
pub struct JSQ4R {
    bits: u8,
}
impl JSQ4R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct JSQ3R {
    bits: u8,
}
impl JSQ3R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct JSQ2R {
    bits: u8,
}
impl JSQ2R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct JSQ1R {
    bits: u8,
}
impl JSQ1R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct JEXTENR {
    bits: u8,
}
impl JEXTENR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct JEXTSELR {
    bits: u8,
}
impl JEXTSELR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct JLR {
    bits: u8,
}
impl JLR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Proxy"]
pub struct _JSQ4W<'a> {
    w: &'a mut W,
}
impl<'a> _JSQ4W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 31;
        const OFFSET: u8 = 26;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _JSQ3W<'a> {
    w: &'a mut W,
}
impl<'a> _JSQ3W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 31;
        const OFFSET: u8 = 20;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _JSQ2W<'a> {
    w: &'a mut W,
}
impl<'a> _JSQ2W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 31;
        const OFFSET: u8 = 14;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _JSQ1W<'a> {
    w: &'a mut W,
}
impl<'a> _JSQ1W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 31;
        const OFFSET: u8 = 8;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _JEXTENW<'a> {
    w: &'a mut W,
}
impl<'a> _JEXTENW<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 6;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _JEXTSELW<'a> {
    w: &'a mut W,
}
impl<'a> _JEXTSELW<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 15;
        const OFFSET: u8 = 2;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _JLW<'a> {
    w: &'a mut W,
}
impl<'a> _JLW<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 0;
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
    #[doc = "Bits 26:30 - JSQ4"]
    #[inline]
    pub fn jsq4(&self) -> JSQ4R {
        let bits = {
            const MASK: u8 = 31;
            const OFFSET: u8 = 26;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        JSQ4R { bits }
    }
    #[doc = "Bits 20:24 - JSQ3"]
    #[inline]
    pub fn jsq3(&self) -> JSQ3R {
        let bits = {
            const MASK: u8 = 31;
            const OFFSET: u8 = 20;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        JSQ3R { bits }
    }
    #[doc = "Bits 14:18 - JSQ2"]
    #[inline]
    pub fn jsq2(&self) -> JSQ2R {
        let bits = {
            const MASK: u8 = 31;
            const OFFSET: u8 = 14;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        JSQ2R { bits }
    }
    #[doc = "Bits 8:12 - JSQ1"]
    #[inline]
    pub fn jsq1(&self) -> JSQ1R {
        let bits = {
            const MASK: u8 = 31;
            const OFFSET: u8 = 8;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        JSQ1R { bits }
    }
    #[doc = "Bits 6:7 - JEXTEN"]
    #[inline]
    pub fn jexten(&self) -> JEXTENR {
        let bits = {
            const MASK: u8 = 3;
            const OFFSET: u8 = 6;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        JEXTENR { bits }
    }
    #[doc = "Bits 2:5 - JEXTSEL"]
    #[inline]
    pub fn jextsel(&self) -> JEXTSELR {
        let bits = {
            const MASK: u8 = 15;
            const OFFSET: u8 = 2;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        JEXTSELR { bits }
    }
    #[doc = "Bits 0:1 - JL"]
    #[inline]
    pub fn jl(&self) -> JLR {
        let bits = {
            const MASK: u8 = 3;
            const OFFSET: u8 = 0;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        JLR { bits }
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
    #[doc = "Bits 26:30 - JSQ4"]
    #[inline]
    pub fn jsq4(&mut self) -> _JSQ4W {
        _JSQ4W { w: self }
    }
    #[doc = "Bits 20:24 - JSQ3"]
    #[inline]
    pub fn jsq3(&mut self) -> _JSQ3W {
        _JSQ3W { w: self }
    }
    #[doc = "Bits 14:18 - JSQ2"]
    #[inline]
    pub fn jsq2(&mut self) -> _JSQ2W {
        _JSQ2W { w: self }
    }
    #[doc = "Bits 8:12 - JSQ1"]
    #[inline]
    pub fn jsq1(&mut self) -> _JSQ1W {
        _JSQ1W { w: self }
    }
    #[doc = "Bits 6:7 - JEXTEN"]
    #[inline]
    pub fn jexten(&mut self) -> _JEXTENW {
        _JEXTENW { w: self }
    }
    #[doc = "Bits 2:5 - JEXTSEL"]
    #[inline]
    pub fn jextsel(&mut self) -> _JEXTSELW {
        _JEXTSELW { w: self }
    }
    #[doc = "Bits 0:1 - JL"]
    #[inline]
    pub fn jl(&mut self) -> _JLW {
        _JLW { w: self }
    }
}
