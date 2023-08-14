#[doc = r" Value to write to the register"]
pub struct W {
    bits: u32,
}
impl super::BSRR {
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
}
#[doc = "Values that can be written to the field `BR15`"]
pub enum BR15W {
    #[doc = "Resets the corresponding ODRx bit"]
    RESET,
}
impl BR15W {
    #[allow(missing_docs)]
    #[doc(hidden)]
    #[inline]
    pub fn _bits(&self) -> bool {
        match *self {
            BR15W::RESET => true,
        }
    }
}
#[doc = r" Proxy"]
pub struct _BR15W<'a> {
    w: &'a mut W,
}
impl<'a> _BR15W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR15W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
#[doc = "Values that can be written to the field `BR14`"]
pub type BR14W = BR15W;
#[doc = r" Proxy"]
pub struct _BR14W<'a> {
    w: &'a mut W,
}
impl<'a> _BR14W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR14W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 30;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR13`"]
pub type BR13W = BR15W;
#[doc = r" Proxy"]
pub struct _BR13W<'a> {
    w: &'a mut W,
}
impl<'a> _BR13W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR13W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 29;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR12`"]
pub type BR12W = BR15W;
#[doc = r" Proxy"]
pub struct _BR12W<'a> {
    w: &'a mut W,
}
impl<'a> _BR12W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR12W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 28;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR11`"]
pub type BR11W = BR15W;
#[doc = r" Proxy"]
pub struct _BR11W<'a> {
    w: &'a mut W,
}
impl<'a> _BR11W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR11W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 27;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR10`"]
pub type BR10W = BR15W;
#[doc = r" Proxy"]
pub struct _BR10W<'a> {
    w: &'a mut W,
}
impl<'a> _BR10W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR10W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 26;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR9`"]
pub type BR9W = BR15W;
#[doc = r" Proxy"]
pub struct _BR9W<'a> {
    w: &'a mut W,
}
impl<'a> _BR9W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR9W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 25;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR8`"]
pub type BR8W = BR15W;
#[doc = r" Proxy"]
pub struct _BR8W<'a> {
    w: &'a mut W,
}
impl<'a> _BR8W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR8W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 24;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR7`"]
pub type BR7W = BR15W;
#[doc = r" Proxy"]
pub struct _BR7W<'a> {
    w: &'a mut W,
}
impl<'a> _BR7W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR7W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 23;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR6`"]
pub type BR6W = BR15W;
#[doc = r" Proxy"]
pub struct _BR6W<'a> {
    w: &'a mut W,
}
impl<'a> _BR6W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR6W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 22;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR5`"]
pub type BR5W = BR15W;
#[doc = r" Proxy"]
pub struct _BR5W<'a> {
    w: &'a mut W,
}
impl<'a> _BR5W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR5W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 21;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR4`"]
pub type BR4W = BR15W;
#[doc = r" Proxy"]
pub struct _BR4W<'a> {
    w: &'a mut W,
}
impl<'a> _BR4W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR4W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 20;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR3`"]
pub type BR3W = BR15W;
#[doc = r" Proxy"]
pub struct _BR3W<'a> {
    w: &'a mut W,
}
impl<'a> _BR3W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR3W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 19;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR2`"]
pub type BR2W = BR15W;
#[doc = r" Proxy"]
pub struct _BR2W<'a> {
    w: &'a mut W,
}
impl<'a> _BR2W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR2W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 18;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR1`"]
pub type BR1W = BR15W;
#[doc = r" Proxy"]
pub struct _BR1W<'a> {
    w: &'a mut W,
}
impl<'a> _BR1W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR1W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 17;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BR0`"]
pub type BR0W = BR15W;
#[doc = r" Proxy"]
pub struct _BR0W<'a> {
    w: &'a mut W,
}
impl<'a> _BR0W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BR0W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Resets the corresponding ODRx bit"]
    #[inline]
    pub fn reset(self) -> &'a mut W {
        self.variant(BR15W::RESET)
    }
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
        const OFFSET: u8 = 16;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BS15`"]
pub enum BS15W {
    #[doc = "Sets the corresponding ODRx bit"]
    SET,
}
impl BS15W {
    #[allow(missing_docs)]
    #[doc(hidden)]
    #[inline]
    pub fn _bits(&self) -> bool {
        match *self {
            BS15W::SET => true,
        }
    }
}
#[doc = r" Proxy"]
pub struct _BS15W<'a> {
    w: &'a mut W,
}
impl<'a> _BS15W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS15W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
#[doc = "Values that can be written to the field `BS14`"]
pub type BS14W = BS15W;
#[doc = r" Proxy"]
pub struct _BS14W<'a> {
    w: &'a mut W,
}
impl<'a> _BS14W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS14W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
        const OFFSET: u8 = 14;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BS13`"]
pub type BS13W = BS15W;
#[doc = r" Proxy"]
pub struct _BS13W<'a> {
    w: &'a mut W,
}
impl<'a> _BS13W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS13W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
        const OFFSET: u8 = 13;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BS12`"]
pub type BS12W = BS15W;
#[doc = r" Proxy"]
pub struct _BS12W<'a> {
    w: &'a mut W,
}
impl<'a> _BS12W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS12W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
        const OFFSET: u8 = 12;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BS11`"]
pub type BS11W = BS15W;
#[doc = r" Proxy"]
pub struct _BS11W<'a> {
    w: &'a mut W,
}
impl<'a> _BS11W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS11W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
        const OFFSET: u8 = 11;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BS10`"]
pub type BS10W = BS15W;
#[doc = r" Proxy"]
pub struct _BS10W<'a> {
    w: &'a mut W,
}
impl<'a> _BS10W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS10W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
        const OFFSET: u8 = 10;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BS9`"]
pub type BS9W = BS15W;
#[doc = r" Proxy"]
pub struct _BS9W<'a> {
    w: &'a mut W,
}
impl<'a> _BS9W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS9W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
#[doc = "Values that can be written to the field `BS8`"]
pub type BS8W = BS15W;
#[doc = r" Proxy"]
pub struct _BS8W<'a> {
    w: &'a mut W,
}
impl<'a> _BS8W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS8W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
        const OFFSET: u8 = 8;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BS7`"]
pub type BS7W = BS15W;
#[doc = r" Proxy"]
pub struct _BS7W<'a> {
    w: &'a mut W,
}
impl<'a> _BS7W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS7W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
#[doc = "Values that can be written to the field `BS6`"]
pub type BS6W = BS15W;
#[doc = r" Proxy"]
pub struct _BS6W<'a> {
    w: &'a mut W,
}
impl<'a> _BS6W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS6W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
        const OFFSET: u8 = 6;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BS5`"]
pub type BS5W = BS15W;
#[doc = r" Proxy"]
pub struct _BS5W<'a> {
    w: &'a mut W,
}
impl<'a> _BS5W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS5W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
        const OFFSET: u8 = 5;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BS4`"]
pub type BS4W = BS15W;
#[doc = r" Proxy"]
pub struct _BS4W<'a> {
    w: &'a mut W,
}
impl<'a> _BS4W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS4W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
        const OFFSET: u8 = 4;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BS3`"]
pub type BS3W = BS15W;
#[doc = r" Proxy"]
pub struct _BS3W<'a> {
    w: &'a mut W,
}
impl<'a> _BS3W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS3W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
        const OFFSET: u8 = 3;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BS2`"]
pub type BS2W = BS15W;
#[doc = r" Proxy"]
pub struct _BS2W<'a> {
    w: &'a mut W,
}
impl<'a> _BS2W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS2W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
        const OFFSET: u8 = 2;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `BS1`"]
pub type BS1W = BS15W;
#[doc = r" Proxy"]
pub struct _BS1W<'a> {
    w: &'a mut W,
}
impl<'a> _BS1W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS1W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
#[doc = "Values that can be written to the field `BS0`"]
pub type BS0W = BS15W;
#[doc = r" Proxy"]
pub struct _BS0W<'a> {
    w: &'a mut W,
}
impl<'a> _BS0W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: BS0W) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Sets the corresponding ODRx bit"]
    #[inline]
    pub fn set(self) -> &'a mut W {
        self.variant(BS15W::SET)
    }
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
    #[doc = "Bit 31 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br15(&mut self) -> _BR15W {
        _BR15W { w: self }
    }
    #[doc = "Bit 30 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br14(&mut self) -> _BR14W {
        _BR14W { w: self }
    }
    #[doc = "Bit 29 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br13(&mut self) -> _BR13W {
        _BR13W { w: self }
    }
    #[doc = "Bit 28 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br12(&mut self) -> _BR12W {
        _BR12W { w: self }
    }
    #[doc = "Bit 27 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br11(&mut self) -> _BR11W {
        _BR11W { w: self }
    }
    #[doc = "Bit 26 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br10(&mut self) -> _BR10W {
        _BR10W { w: self }
    }
    #[doc = "Bit 25 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br9(&mut self) -> _BR9W {
        _BR9W { w: self }
    }
    #[doc = "Bit 24 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br8(&mut self) -> _BR8W {
        _BR8W { w: self }
    }
    #[doc = "Bit 23 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br7(&mut self) -> _BR7W {
        _BR7W { w: self }
    }
    #[doc = "Bit 22 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br6(&mut self) -> _BR6W {
        _BR6W { w: self }
    }
    #[doc = "Bit 21 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br5(&mut self) -> _BR5W {
        _BR5W { w: self }
    }
    #[doc = "Bit 20 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br4(&mut self) -> _BR4W {
        _BR4W { w: self }
    }
    #[doc = "Bit 19 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br3(&mut self) -> _BR3W {
        _BR3W { w: self }
    }
    #[doc = "Bit 18 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br2(&mut self) -> _BR2W {
        _BR2W { w: self }
    }
    #[doc = "Bit 17 - Port x reset bit y (y = 0..15)"]
    #[inline]
    pub fn br1(&mut self) -> _BR1W {
        _BR1W { w: self }
    }
    #[doc = "Bit 16 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn br0(&mut self) -> _BR0W {
        _BR0W { w: self }
    }
    #[doc = "Bit 15 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs15(&mut self) -> _BS15W {
        _BS15W { w: self }
    }
    #[doc = "Bit 14 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs14(&mut self) -> _BS14W {
        _BS14W { w: self }
    }
    #[doc = "Bit 13 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs13(&mut self) -> _BS13W {
        _BS13W { w: self }
    }
    #[doc = "Bit 12 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs12(&mut self) -> _BS12W {
        _BS12W { w: self }
    }
    #[doc = "Bit 11 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs11(&mut self) -> _BS11W {
        _BS11W { w: self }
    }
    #[doc = "Bit 10 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs10(&mut self) -> _BS10W {
        _BS10W { w: self }
    }
    #[doc = "Bit 9 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs9(&mut self) -> _BS9W {
        _BS9W { w: self }
    }
    #[doc = "Bit 8 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs8(&mut self) -> _BS8W {
        _BS8W { w: self }
    }
    #[doc = "Bit 7 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs7(&mut self) -> _BS7W {
        _BS7W { w: self }
    }
    #[doc = "Bit 6 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs6(&mut self) -> _BS6W {
        _BS6W { w: self }
    }
    #[doc = "Bit 5 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs5(&mut self) -> _BS5W {
        _BS5W { w: self }
    }
    #[doc = "Bit 4 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs4(&mut self) -> _BS4W {
        _BS4W { w: self }
    }
    #[doc = "Bit 3 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs3(&mut self) -> _BS3W {
        _BS3W { w: self }
    }
    #[doc = "Bit 2 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs2(&mut self) -> _BS2W {
        _BS2W { w: self }
    }
    #[doc = "Bit 1 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs1(&mut self) -> _BS1W {
        _BS1W { w: self }
    }
    #[doc = "Bit 0 - Port x set bit y (y= 0..15)"]
    #[inline]
    pub fn bs0(&mut self) -> _BS0W {
        _BS0W { w: self }
    }
}
