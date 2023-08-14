#[doc = r" Value read from the register"]
pub struct R {
    bits: u32,
}
#[doc = r" Value to write to the register"]
pub struct W {
    bits: u32,
}
impl super::OR {
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
pub struct TIM1_ETR_ADC1_RMPR {
    bits: u8,
}
impl TIM1_ETR_ADC1_RMPR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct TIM1_ETR_ADC4_RMPR {
    bits: u8,
}
impl TIM1_ETR_ADC4_RMPR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Proxy"]
pub struct _TIM1_ETR_ADC1_RMPW<'a> {
    w: &'a mut W,
}
impl<'a> _TIM1_ETR_ADC1_RMPW<'a> {
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
#[doc = r" Proxy"]
pub struct _TIM1_ETR_ADC4_RMPW<'a> {
    w: &'a mut W,
}
impl<'a> _TIM1_ETR_ADC4_RMPW<'a> {
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
impl R {
    #[doc = r" Value of the register as raw bits"]
    #[inline]
    pub fn bits(&self) -> u32 {
        self.bits
    }
    #[doc = "Bits 0:1 - TIM1_ETR_ADC1 remapping capability"]
    #[inline]
    pub fn tim1_etr_adc1_rmp(&self) -> TIM1_ETR_ADC1_RMPR {
        let bits = {
            const MASK: u8 = 3;
            const OFFSET: u8 = 0;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        TIM1_ETR_ADC1_RMPR { bits }
    }
    #[doc = "Bits 2:3 - TIM1_ETR_ADC4 remapping capability"]
    #[inline]
    pub fn tim1_etr_adc4_rmp(&self) -> TIM1_ETR_ADC4_RMPR {
        let bits = {
            const MASK: u8 = 3;
            const OFFSET: u8 = 2;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        TIM1_ETR_ADC4_RMPR { bits }
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
    #[doc = "Bits 0:1 - TIM1_ETR_ADC1 remapping capability"]
    #[inline]
    pub fn tim1_etr_adc1_rmp(&mut self) -> _TIM1_ETR_ADC1_RMPW {
        _TIM1_ETR_ADC1_RMPW { w: self }
    }
    #[doc = "Bits 2:3 - TIM1_ETR_ADC4 remapping capability"]
    #[inline]
    pub fn tim1_etr_adc4_rmp(&mut self) -> _TIM1_ETR_ADC4_RMPW {
        _TIM1_ETR_ADC4_RMPW { w: self }
    }
}
