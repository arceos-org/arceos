#[doc = r" Value read from the register"]
pub struct R {
    bits: u32,
}
#[doc = r" Value to write to the register"]
pub struct W {
    bits: u32,
}
impl super::AFRL {
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
pub struct AFRL7R {
    bits: u8,
}
impl AFRL7R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct AFRL6R {
    bits: u8,
}
impl AFRL6R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct AFRL5R {
    bits: u8,
}
impl AFRL5R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct AFRL4R {
    bits: u8,
}
impl AFRL4R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct AFRL3R {
    bits: u8,
}
impl AFRL3R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct AFRL2R {
    bits: u8,
}
impl AFRL2R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct AFRL1R {
    bits: u8,
}
impl AFRL1R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct AFRL0R {
    bits: u8,
}
impl AFRL0R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Proxy"]
pub struct _AFRL7W<'a> {
    w: &'a mut W,
}
impl<'a> _AFRL7W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 15;
        const OFFSET: u8 = 28;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _AFRL6W<'a> {
    w: &'a mut W,
}
impl<'a> _AFRL6W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 15;
        const OFFSET: u8 = 24;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _AFRL5W<'a> {
    w: &'a mut W,
}
impl<'a> _AFRL5W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 15;
        const OFFSET: u8 = 20;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _AFRL4W<'a> {
    w: &'a mut W,
}
impl<'a> _AFRL4W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 15;
        const OFFSET: u8 = 16;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _AFRL3W<'a> {
    w: &'a mut W,
}
impl<'a> _AFRL3W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 15;
        const OFFSET: u8 = 12;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _AFRL2W<'a> {
    w: &'a mut W,
}
impl<'a> _AFRL2W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 15;
        const OFFSET: u8 = 8;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _AFRL1W<'a> {
    w: &'a mut W,
}
impl<'a> _AFRL1W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 15;
        const OFFSET: u8 = 4;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _AFRL0W<'a> {
    w: &'a mut W,
}
impl<'a> _AFRL0W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 15;
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
    #[doc = "Bits 28:31 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl7(&self) -> AFRL7R {
        let bits = {
            const MASK: u8 = 15;
            const OFFSET: u8 = 28;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        AFRL7R { bits }
    }
    #[doc = "Bits 24:27 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl6(&self) -> AFRL6R {
        let bits = {
            const MASK: u8 = 15;
            const OFFSET: u8 = 24;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        AFRL6R { bits }
    }
    #[doc = "Bits 20:23 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl5(&self) -> AFRL5R {
        let bits = {
            const MASK: u8 = 15;
            const OFFSET: u8 = 20;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        AFRL5R { bits }
    }
    #[doc = "Bits 16:19 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl4(&self) -> AFRL4R {
        let bits = {
            const MASK: u8 = 15;
            const OFFSET: u8 = 16;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        AFRL4R { bits }
    }
    #[doc = "Bits 12:15 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl3(&self) -> AFRL3R {
        let bits = {
            const MASK: u8 = 15;
            const OFFSET: u8 = 12;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        AFRL3R { bits }
    }
    #[doc = "Bits 8:11 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl2(&self) -> AFRL2R {
        let bits = {
            const MASK: u8 = 15;
            const OFFSET: u8 = 8;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        AFRL2R { bits }
    }
    #[doc = "Bits 4:7 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl1(&self) -> AFRL1R {
        let bits = {
            const MASK: u8 = 15;
            const OFFSET: u8 = 4;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        AFRL1R { bits }
    }
    #[doc = "Bits 0:3 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl0(&self) -> AFRL0R {
        let bits = {
            const MASK: u8 = 15;
            const OFFSET: u8 = 0;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        AFRL0R { bits }
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
    #[doc = "Bits 28:31 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl7(&mut self) -> _AFRL7W {
        _AFRL7W { w: self }
    }
    #[doc = "Bits 24:27 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl6(&mut self) -> _AFRL6W {
        _AFRL6W { w: self }
    }
    #[doc = "Bits 20:23 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl5(&mut self) -> _AFRL5W {
        _AFRL5W { w: self }
    }
    #[doc = "Bits 16:19 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl4(&mut self) -> _AFRL4W {
        _AFRL4W { w: self }
    }
    #[doc = "Bits 12:15 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl3(&mut self) -> _AFRL3W {
        _AFRL3W { w: self }
    }
    #[doc = "Bits 8:11 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl2(&mut self) -> _AFRL2W {
        _AFRL2W { w: self }
    }
    #[doc = "Bits 4:7 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl1(&mut self) -> _AFRL1W {
        _AFRL1W { w: self }
    }
    #[doc = "Bits 0:3 - Alternate function selection for port x bit y (y = 0..7)"]
    #[inline]
    pub fn afrl0(&mut self) -> _AFRL0W {
        _AFRL0W { w: self }
    }
}
