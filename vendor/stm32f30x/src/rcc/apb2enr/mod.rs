#[doc = r" Value read from the register"]
pub struct R {
    bits: u32,
}
#[doc = r" Value to write to the register"]
pub struct W {
    bits: u32,
}
impl super::APB2ENR {
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
#[doc = "Possible values of the field `SYSCFGEN`"]
pub type SYSCFGENR = super::ahbenr::DMAENR;
#[doc = "Possible values of the field `TIM1EN`"]
pub type TIM1ENR = super::ahbenr::DMAENR;
#[doc = "Possible values of the field `SPI1EN`"]
pub type SPI1ENR = super::ahbenr::DMAENR;
#[doc = "Possible values of the field `TIM8EN`"]
pub type TIM8ENR = super::ahbenr::DMAENR;
#[doc = "Possible values of the field `USART1EN`"]
pub type USART1ENR = super::ahbenr::DMAENR;
#[doc = "Possible values of the field `TIM15EN`"]
pub type TIM15ENR = super::ahbenr::DMAENR;
#[doc = "Possible values of the field `TIM16EN`"]
pub type TIM16ENR = super::ahbenr::DMAENR;
#[doc = "Possible values of the field `TIM17EN`"]
pub type TIM17ENR = super::ahbenr::DMAENR;
#[doc = "Values that can be written to the field `SYSCFGEN`"]
pub type SYSCFGENW = super::ahbenr::DMAENW;
#[doc = r" Proxy"]
pub struct _SYSCFGENW<'a> {
    w: &'a mut W,
}
impl<'a> _SYSCFGENW<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: SYSCFGENW) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Disabled."]
    #[inline]
    pub fn disabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::DISABLED)
    }
    #[doc = "Enabled."]
    #[inline]
    pub fn enabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::ENABLED)
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
#[doc = "Values that can be written to the field `TIM1EN`"]
pub type TIM1ENW = super::ahbenr::DMAENW;
#[doc = r" Proxy"]
pub struct _TIM1ENW<'a> {
    w: &'a mut W,
}
impl<'a> _TIM1ENW<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: TIM1ENW) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Disabled."]
    #[inline]
    pub fn disabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::DISABLED)
    }
    #[doc = "Enabled."]
    #[inline]
    pub fn enabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::ENABLED)
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
#[doc = "Values that can be written to the field `SPI1EN`"]
pub type SPI1ENW = super::ahbenr::DMAENW;
#[doc = r" Proxy"]
pub struct _SPI1ENW<'a> {
    w: &'a mut W,
}
impl<'a> _SPI1ENW<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: SPI1ENW) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Disabled."]
    #[inline]
    pub fn disabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::DISABLED)
    }
    #[doc = "Enabled."]
    #[inline]
    pub fn enabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::ENABLED)
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
#[doc = "Values that can be written to the field `TIM8EN`"]
pub type TIM8ENW = super::ahbenr::DMAENW;
#[doc = r" Proxy"]
pub struct _TIM8ENW<'a> {
    w: &'a mut W,
}
impl<'a> _TIM8ENW<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: TIM8ENW) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Disabled."]
    #[inline]
    pub fn disabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::DISABLED)
    }
    #[doc = "Enabled."]
    #[inline]
    pub fn enabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::ENABLED)
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
#[doc = "Values that can be written to the field `USART1EN`"]
pub type USART1ENW = super::ahbenr::DMAENW;
#[doc = r" Proxy"]
pub struct _USART1ENW<'a> {
    w: &'a mut W,
}
impl<'a> _USART1ENW<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: USART1ENW) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Disabled."]
    #[inline]
    pub fn disabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::DISABLED)
    }
    #[doc = "Enabled."]
    #[inline]
    pub fn enabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::ENABLED)
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
#[doc = "Values that can be written to the field `TIM15EN`"]
pub type TIM15ENW = super::ahbenr::DMAENW;
#[doc = r" Proxy"]
pub struct _TIM15ENW<'a> {
    w: &'a mut W,
}
impl<'a> _TIM15ENW<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: TIM15ENW) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Disabled."]
    #[inline]
    pub fn disabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::DISABLED)
    }
    #[doc = "Enabled."]
    #[inline]
    pub fn enabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::ENABLED)
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
#[doc = "Values that can be written to the field `TIM16EN`"]
pub type TIM16ENW = super::ahbenr::DMAENW;
#[doc = r" Proxy"]
pub struct _TIM16ENW<'a> {
    w: &'a mut W,
}
impl<'a> _TIM16ENW<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: TIM16ENW) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Disabled."]
    #[inline]
    pub fn disabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::DISABLED)
    }
    #[doc = "Enabled."]
    #[inline]
    pub fn enabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::ENABLED)
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
#[doc = "Values that can be written to the field `TIM17EN`"]
pub type TIM17ENW = super::ahbenr::DMAENW;
#[doc = r" Proxy"]
pub struct _TIM17ENW<'a> {
    w: &'a mut W,
}
impl<'a> _TIM17ENW<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: TIM17ENW) -> &'a mut W {
        {
            self.bit(variant._bits())
        }
    }
    #[doc = "Disabled."]
    #[inline]
    pub fn disabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::DISABLED)
    }
    #[doc = "Enabled."]
    #[inline]
    pub fn enabled(self) -> &'a mut W {
        self.variant(super::ahbenr::DMAENW::ENABLED)
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
impl R {
    #[doc = r" Value of the register as raw bits"]
    #[inline]
    pub fn bits(&self) -> u32 {
        self.bits
    }
    #[doc = "Bit 0 - SYSCFG clock enable"]
    #[inline]
    pub fn syscfgen(&self) -> SYSCFGENR {
        SYSCFGENR::_from({
            const MASK: bool = true;
            const OFFSET: u8 = 0;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        })
    }
    #[doc = "Bit 11 - TIM1 Timer clock enable"]
    #[inline]
    pub fn tim1en(&self) -> TIM1ENR {
        TIM1ENR::_from({
            const MASK: bool = true;
            const OFFSET: u8 = 11;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        })
    }
    #[doc = "Bit 12 - SPI 1 clock enable"]
    #[inline]
    pub fn spi1en(&self) -> SPI1ENR {
        SPI1ENR::_from({
            const MASK: bool = true;
            const OFFSET: u8 = 12;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        })
    }
    #[doc = "Bit 13 - TIM8 Timer clock enable"]
    #[inline]
    pub fn tim8en(&self) -> TIM8ENR {
        TIM8ENR::_from({
            const MASK: bool = true;
            const OFFSET: u8 = 13;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        })
    }
    #[doc = "Bit 14 - USART1 clock enable"]
    #[inline]
    pub fn usart1en(&self) -> USART1ENR {
        USART1ENR::_from({
            const MASK: bool = true;
            const OFFSET: u8 = 14;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        })
    }
    #[doc = "Bit 16 - TIM15 timer clock enable"]
    #[inline]
    pub fn tim15en(&self) -> TIM15ENR {
        TIM15ENR::_from({
            const MASK: bool = true;
            const OFFSET: u8 = 16;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        })
    }
    #[doc = "Bit 17 - TIM16 timer clock enable"]
    #[inline]
    pub fn tim16en(&self) -> TIM16ENR {
        TIM16ENR::_from({
            const MASK: bool = true;
            const OFFSET: u8 = 17;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        })
    }
    #[doc = "Bit 18 - TIM17 timer clock enable"]
    #[inline]
    pub fn tim17en(&self) -> TIM17ENR {
        TIM17ENR::_from({
            const MASK: bool = true;
            const OFFSET: u8 = 18;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        })
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
    #[doc = "Bit 0 - SYSCFG clock enable"]
    #[inline]
    pub fn syscfgen(&mut self) -> _SYSCFGENW {
        _SYSCFGENW { w: self }
    }
    #[doc = "Bit 11 - TIM1 Timer clock enable"]
    #[inline]
    pub fn tim1en(&mut self) -> _TIM1ENW {
        _TIM1ENW { w: self }
    }
    #[doc = "Bit 12 - SPI 1 clock enable"]
    #[inline]
    pub fn spi1en(&mut self) -> _SPI1ENW {
        _SPI1ENW { w: self }
    }
    #[doc = "Bit 13 - TIM8 Timer clock enable"]
    #[inline]
    pub fn tim8en(&mut self) -> _TIM8ENW {
        _TIM8ENW { w: self }
    }
    #[doc = "Bit 14 - USART1 clock enable"]
    #[inline]
    pub fn usart1en(&mut self) -> _USART1ENW {
        _USART1ENW { w: self }
    }
    #[doc = "Bit 16 - TIM15 timer clock enable"]
    #[inline]
    pub fn tim15en(&mut self) -> _TIM15ENW {
        _TIM15ENW { w: self }
    }
    #[doc = "Bit 17 - TIM16 timer clock enable"]
    #[inline]
    pub fn tim16en(&mut self) -> _TIM16ENW {
        _TIM16ENW { w: self }
    }
    #[doc = "Bit 18 - TIM17 timer clock enable"]
    #[inline]
    pub fn tim17en(&mut self) -> _TIM17ENW {
        _TIM17ENW { w: self }
    }
}
