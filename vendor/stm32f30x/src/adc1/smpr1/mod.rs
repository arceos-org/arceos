#[doc = r" Value read from the register"]
pub struct R {
    bits: u32,
}
#[doc = r" Value to write to the register"]
pub struct W {
    bits: u32,
}
impl super::SMPR1 {
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
pub struct SMP9R {
    bits: u8,
}
impl SMP9R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct SMP8R {
    bits: u8,
}
impl SMP8R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct SMP7R {
    bits: u8,
}
impl SMP7R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct SMP6R {
    bits: u8,
}
impl SMP6R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct SMP5R {
    bits: u8,
}
impl SMP5R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct SMP4R {
    bits: u8,
}
impl SMP4R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct SMP3R {
    bits: u8,
}
impl SMP3R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct SMP2R {
    bits: u8,
}
impl SMP2R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct SMP1R {
    bits: u8,
}
impl SMP1R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Proxy"]
pub struct _SMP9W<'a> {
    w: &'a mut W,
}
impl<'a> _SMP9W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 7;
        const OFFSET: u8 = 27;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _SMP8W<'a> {
    w: &'a mut W,
}
impl<'a> _SMP8W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 7;
        const OFFSET: u8 = 24;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _SMP7W<'a> {
    w: &'a mut W,
}
impl<'a> _SMP7W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 7;
        const OFFSET: u8 = 21;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _SMP6W<'a> {
    w: &'a mut W,
}
impl<'a> _SMP6W<'a> {
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
pub struct _SMP5W<'a> {
    w: &'a mut W,
}
impl<'a> _SMP5W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 7;
        const OFFSET: u8 = 15;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _SMP4W<'a> {
    w: &'a mut W,
}
impl<'a> _SMP4W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 7;
        const OFFSET: u8 = 12;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _SMP3W<'a> {
    w: &'a mut W,
}
impl<'a> _SMP3W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 7;
        const OFFSET: u8 = 9;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _SMP2W<'a> {
    w: &'a mut W,
}
impl<'a> _SMP2W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 7;
        const OFFSET: u8 = 6;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _SMP1W<'a> {
    w: &'a mut W,
}
impl<'a> _SMP1W<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 7;
        const OFFSET: u8 = 3;
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
    #[doc = "Bits 27:29 - SMP9"]
    #[inline]
    pub fn smp9(&self) -> SMP9R {
        let bits = {
            const MASK: u8 = 7;
            const OFFSET: u8 = 27;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        SMP9R { bits }
    }
    #[doc = "Bits 24:26 - SMP8"]
    #[inline]
    pub fn smp8(&self) -> SMP8R {
        let bits = {
            const MASK: u8 = 7;
            const OFFSET: u8 = 24;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        SMP8R { bits }
    }
    #[doc = "Bits 21:23 - SMP7"]
    #[inline]
    pub fn smp7(&self) -> SMP7R {
        let bits = {
            const MASK: u8 = 7;
            const OFFSET: u8 = 21;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        SMP7R { bits }
    }
    #[doc = "Bits 18:20 - SMP6"]
    #[inline]
    pub fn smp6(&self) -> SMP6R {
        let bits = {
            const MASK: u8 = 7;
            const OFFSET: u8 = 18;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        SMP6R { bits }
    }
    #[doc = "Bits 15:17 - SMP5"]
    #[inline]
    pub fn smp5(&self) -> SMP5R {
        let bits = {
            const MASK: u8 = 7;
            const OFFSET: u8 = 15;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        SMP5R { bits }
    }
    #[doc = "Bits 12:14 - SMP4"]
    #[inline]
    pub fn smp4(&self) -> SMP4R {
        let bits = {
            const MASK: u8 = 7;
            const OFFSET: u8 = 12;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        SMP4R { bits }
    }
    #[doc = "Bits 9:11 - SMP3"]
    #[inline]
    pub fn smp3(&self) -> SMP3R {
        let bits = {
            const MASK: u8 = 7;
            const OFFSET: u8 = 9;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        SMP3R { bits }
    }
    #[doc = "Bits 6:8 - SMP2"]
    #[inline]
    pub fn smp2(&self) -> SMP2R {
        let bits = {
            const MASK: u8 = 7;
            const OFFSET: u8 = 6;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        SMP2R { bits }
    }
    #[doc = "Bits 3:5 - SMP1"]
    #[inline]
    pub fn smp1(&self) -> SMP1R {
        let bits = {
            const MASK: u8 = 7;
            const OFFSET: u8 = 3;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        SMP1R { bits }
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
    #[doc = "Bits 27:29 - SMP9"]
    #[inline]
    pub fn smp9(&mut self) -> _SMP9W {
        _SMP9W { w: self }
    }
    #[doc = "Bits 24:26 - SMP8"]
    #[inline]
    pub fn smp8(&mut self) -> _SMP8W {
        _SMP8W { w: self }
    }
    #[doc = "Bits 21:23 - SMP7"]
    #[inline]
    pub fn smp7(&mut self) -> _SMP7W {
        _SMP7W { w: self }
    }
    #[doc = "Bits 18:20 - SMP6"]
    #[inline]
    pub fn smp6(&mut self) -> _SMP6W {
        _SMP6W { w: self }
    }
    #[doc = "Bits 15:17 - SMP5"]
    #[inline]
    pub fn smp5(&mut self) -> _SMP5W {
        _SMP5W { w: self }
    }
    #[doc = "Bits 12:14 - SMP4"]
    #[inline]
    pub fn smp4(&mut self) -> _SMP4W {
        _SMP4W { w: self }
    }
    #[doc = "Bits 9:11 - SMP3"]
    #[inline]
    pub fn smp3(&mut self) -> _SMP3W {
        _SMP3W { w: self }
    }
    #[doc = "Bits 6:8 - SMP2"]
    #[inline]
    pub fn smp2(&mut self) -> _SMP2W {
        _SMP2W { w: self }
    }
    #[doc = "Bits 3:5 - SMP1"]
    #[inline]
    pub fn smp1(&mut self) -> _SMP1W {
        _SMP1W { w: self }
    }
}
