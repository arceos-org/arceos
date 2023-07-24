#[doc = r" Value read from the register"]
pub struct R {
    bits: u32,
}
#[doc = r" Value to write to the register"]
pub struct W {
    bits: u32,
}
impl super::CFGR1 {
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
pub struct MEM_MODER {
    bits: u8,
}
impl MEM_MODER {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct USB_IT_RMPR {
    bits: bool,
}
impl USB_IT_RMPR {
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
pub struct TIM1_ITR_RMPR {
    bits: bool,
}
impl TIM1_ITR_RMPR {
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
pub struct DAC_TRIG_RMPR {
    bits: bool,
}
impl DAC_TRIG_RMPR {
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
pub struct ADC24_DMA_RMPR {
    bits: bool,
}
impl ADC24_DMA_RMPR {
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
pub struct TIM16_DMA_RMPR {
    bits: bool,
}
impl TIM16_DMA_RMPR {
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
pub struct TIM17_DMA_RMPR {
    bits: bool,
}
impl TIM17_DMA_RMPR {
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
pub struct TIM6_DAC1_DMA_RMPR {
    bits: bool,
}
impl TIM6_DAC1_DMA_RMPR {
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
pub struct TIM7_DAC2_DMA_RMPR {
    bits: bool,
}
impl TIM7_DAC2_DMA_RMPR {
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
pub struct I2C_PB6_FMR {
    bits: bool,
}
impl I2C_PB6_FMR {
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
pub struct I2C_PB7_FMR {
    bits: bool,
}
impl I2C_PB7_FMR {
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
pub struct I2C_PB8_FMR {
    bits: bool,
}
impl I2C_PB8_FMR {
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
pub struct I2C_PB9_FMR {
    bits: bool,
}
impl I2C_PB9_FMR {
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
pub struct I2C1_FMR {
    bits: bool,
}
impl I2C1_FMR {
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
pub struct I2C2_FMR {
    bits: bool,
}
impl I2C2_FMR {
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
pub struct ENCODER_MODER {
    bits: u8,
}
impl ENCODER_MODER {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Value of the field"]
pub struct FPU_ITR {
    bits: u8,
}
impl FPU_ITR {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        self.bits
    }
}
#[doc = r" Proxy"]
pub struct _MEM_MODEW<'a> {
    w: &'a mut W,
}
impl<'a> _MEM_MODEW<'a> {
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
pub struct _USB_IT_RMPW<'a> {
    w: &'a mut W,
}
impl<'a> _USB_IT_RMPW<'a> {
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
#[doc = r" Proxy"]
pub struct _TIM1_ITR_RMPW<'a> {
    w: &'a mut W,
}
impl<'a> _TIM1_ITR_RMPW<'a> {
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
#[doc = r" Proxy"]
pub struct _DAC_TRIG_RMPW<'a> {
    w: &'a mut W,
}
impl<'a> _DAC_TRIG_RMPW<'a> {
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
pub struct _ADC24_DMA_RMPW<'a> {
    w: &'a mut W,
}
impl<'a> _ADC24_DMA_RMPW<'a> {
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
#[doc = r" Proxy"]
pub struct _TIM16_DMA_RMPW<'a> {
    w: &'a mut W,
}
impl<'a> _TIM16_DMA_RMPW<'a> {
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
#[doc = r" Proxy"]
pub struct _TIM17_DMA_RMPW<'a> {
    w: &'a mut W,
}
impl<'a> _TIM17_DMA_RMPW<'a> {
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
#[doc = r" Proxy"]
pub struct _TIM6_DAC1_DMA_RMPW<'a> {
    w: &'a mut W,
}
impl<'a> _TIM6_DAC1_DMA_RMPW<'a> {
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
#[doc = r" Proxy"]
pub struct _TIM7_DAC2_DMA_RMPW<'a> {
    w: &'a mut W,
}
impl<'a> _TIM7_DAC2_DMA_RMPW<'a> {
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
#[doc = r" Proxy"]
pub struct _I2C_PB6_FMW<'a> {
    w: &'a mut W,
}
impl<'a> _I2C_PB6_FMW<'a> {
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
#[doc = r" Proxy"]
pub struct _I2C_PB7_FMW<'a> {
    w: &'a mut W,
}
impl<'a> _I2C_PB7_FMW<'a> {
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
#[doc = r" Proxy"]
pub struct _I2C_PB8_FMW<'a> {
    w: &'a mut W,
}
impl<'a> _I2C_PB8_FMW<'a> {
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
#[doc = r" Proxy"]
pub struct _I2C_PB9_FMW<'a> {
    w: &'a mut W,
}
impl<'a> _I2C_PB9_FMW<'a> {
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
#[doc = r" Proxy"]
pub struct _I2C1_FMW<'a> {
    w: &'a mut W,
}
impl<'a> _I2C1_FMW<'a> {
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
#[doc = r" Proxy"]
pub struct _I2C2_FMW<'a> {
    w: &'a mut W,
}
impl<'a> _I2C2_FMW<'a> {
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
#[doc = r" Proxy"]
pub struct _ENCODER_MODEW<'a> {
    w: &'a mut W,
}
impl<'a> _ENCODER_MODEW<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 22;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = r" Proxy"]
pub struct _FPU_ITW<'a> {
    w: &'a mut W,
}
impl<'a> _FPU_ITW<'a> {
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 63;
        const OFFSET: u8 = 26;
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
    #[doc = "Bits 0:1 - Memory mapping selection bits"]
    #[inline]
    pub fn mem_mode(&self) -> MEM_MODER {
        let bits = {
            const MASK: u8 = 3;
            const OFFSET: u8 = 0;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        MEM_MODER { bits }
    }
    #[doc = "Bit 5 - USB interrupt remap"]
    #[inline]
    pub fn usb_it_rmp(&self) -> USB_IT_RMPR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 5;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        USB_IT_RMPR { bits }
    }
    #[doc = "Bit 6 - Timer 1 ITR3 selection"]
    #[inline]
    pub fn tim1_itr_rmp(&self) -> TIM1_ITR_RMPR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 6;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        TIM1_ITR_RMPR { bits }
    }
    #[doc = "Bit 7 - DAC trigger remap (when TSEL = 001)"]
    #[inline]
    pub fn dac_trig_rmp(&self) -> DAC_TRIG_RMPR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 7;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        DAC_TRIG_RMPR { bits }
    }
    #[doc = "Bit 8 - ADC24 DMA remapping bit"]
    #[inline]
    pub fn adc24_dma_rmp(&self) -> ADC24_DMA_RMPR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 8;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        ADC24_DMA_RMPR { bits }
    }
    #[doc = "Bit 11 - TIM16 DMA request remapping bit"]
    #[inline]
    pub fn tim16_dma_rmp(&self) -> TIM16_DMA_RMPR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 11;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        TIM16_DMA_RMPR { bits }
    }
    #[doc = "Bit 12 - TIM17 DMA request remapping bit"]
    #[inline]
    pub fn tim17_dma_rmp(&self) -> TIM17_DMA_RMPR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 12;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        TIM17_DMA_RMPR { bits }
    }
    #[doc = "Bit 13 - TIM6 and DAC1 DMA request remapping bit"]
    #[inline]
    pub fn tim6_dac1_dma_rmp(&self) -> TIM6_DAC1_DMA_RMPR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 13;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        TIM6_DAC1_DMA_RMPR { bits }
    }
    #[doc = "Bit 14 - TIM7 and DAC2 DMA request remapping bit"]
    #[inline]
    pub fn tim7_dac2_dma_rmp(&self) -> TIM7_DAC2_DMA_RMPR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 14;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        TIM7_DAC2_DMA_RMPR { bits }
    }
    #[doc = "Bit 16 - Fast Mode Plus (FM+) driving capability activation bits."]
    #[inline]
    pub fn i2c_pb6_fm(&self) -> I2C_PB6_FMR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 16;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        I2C_PB6_FMR { bits }
    }
    #[doc = "Bit 17 - Fast Mode Plus (FM+) driving capability activation bits."]
    #[inline]
    pub fn i2c_pb7_fm(&self) -> I2C_PB7_FMR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 17;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        I2C_PB7_FMR { bits }
    }
    #[doc = "Bit 18 - Fast Mode Plus (FM+) driving capability activation bits."]
    #[inline]
    pub fn i2c_pb8_fm(&self) -> I2C_PB8_FMR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 18;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        I2C_PB8_FMR { bits }
    }
    #[doc = "Bit 19 - Fast Mode Plus (FM+) driving capability activation bits."]
    #[inline]
    pub fn i2c_pb9_fm(&self) -> I2C_PB9_FMR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 19;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        I2C_PB9_FMR { bits }
    }
    #[doc = "Bit 20 - I2C1 Fast Mode Plus"]
    #[inline]
    pub fn i2c1_fm(&self) -> I2C1_FMR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 20;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        I2C1_FMR { bits }
    }
    #[doc = "Bit 21 - I2C2 Fast Mode Plus"]
    #[inline]
    pub fn i2c2_fm(&self) -> I2C2_FMR {
        let bits = {
            const MASK: bool = true;
            const OFFSET: u8 = 21;
            ((self.bits >> OFFSET) & MASK as u32) != 0
        };
        I2C2_FMR { bits }
    }
    #[doc = "Bits 22:23 - Encoder mode"]
    #[inline]
    pub fn encoder_mode(&self) -> ENCODER_MODER {
        let bits = {
            const MASK: u8 = 3;
            const OFFSET: u8 = 22;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        ENCODER_MODER { bits }
    }
    #[doc = "Bits 26:31 - Interrupt enable bits from FPU"]
    #[inline]
    pub fn fpu_it(&self) -> FPU_ITR {
        let bits = {
            const MASK: u8 = 63;
            const OFFSET: u8 = 26;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        };
        FPU_ITR { bits }
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
    #[doc = "Bits 0:1 - Memory mapping selection bits"]
    #[inline]
    pub fn mem_mode(&mut self) -> _MEM_MODEW {
        _MEM_MODEW { w: self }
    }
    #[doc = "Bit 5 - USB interrupt remap"]
    #[inline]
    pub fn usb_it_rmp(&mut self) -> _USB_IT_RMPW {
        _USB_IT_RMPW { w: self }
    }
    #[doc = "Bit 6 - Timer 1 ITR3 selection"]
    #[inline]
    pub fn tim1_itr_rmp(&mut self) -> _TIM1_ITR_RMPW {
        _TIM1_ITR_RMPW { w: self }
    }
    #[doc = "Bit 7 - DAC trigger remap (when TSEL = 001)"]
    #[inline]
    pub fn dac_trig_rmp(&mut self) -> _DAC_TRIG_RMPW {
        _DAC_TRIG_RMPW { w: self }
    }
    #[doc = "Bit 8 - ADC24 DMA remapping bit"]
    #[inline]
    pub fn adc24_dma_rmp(&mut self) -> _ADC24_DMA_RMPW {
        _ADC24_DMA_RMPW { w: self }
    }
    #[doc = "Bit 11 - TIM16 DMA request remapping bit"]
    #[inline]
    pub fn tim16_dma_rmp(&mut self) -> _TIM16_DMA_RMPW {
        _TIM16_DMA_RMPW { w: self }
    }
    #[doc = "Bit 12 - TIM17 DMA request remapping bit"]
    #[inline]
    pub fn tim17_dma_rmp(&mut self) -> _TIM17_DMA_RMPW {
        _TIM17_DMA_RMPW { w: self }
    }
    #[doc = "Bit 13 - TIM6 and DAC1 DMA request remapping bit"]
    #[inline]
    pub fn tim6_dac1_dma_rmp(&mut self) -> _TIM6_DAC1_DMA_RMPW {
        _TIM6_DAC1_DMA_RMPW { w: self }
    }
    #[doc = "Bit 14 - TIM7 and DAC2 DMA request remapping bit"]
    #[inline]
    pub fn tim7_dac2_dma_rmp(&mut self) -> _TIM7_DAC2_DMA_RMPW {
        _TIM7_DAC2_DMA_RMPW { w: self }
    }
    #[doc = "Bit 16 - Fast Mode Plus (FM+) driving capability activation bits."]
    #[inline]
    pub fn i2c_pb6_fm(&mut self) -> _I2C_PB6_FMW {
        _I2C_PB6_FMW { w: self }
    }
    #[doc = "Bit 17 - Fast Mode Plus (FM+) driving capability activation bits."]
    #[inline]
    pub fn i2c_pb7_fm(&mut self) -> _I2C_PB7_FMW {
        _I2C_PB7_FMW { w: self }
    }
    #[doc = "Bit 18 - Fast Mode Plus (FM+) driving capability activation bits."]
    #[inline]
    pub fn i2c_pb8_fm(&mut self) -> _I2C_PB8_FMW {
        _I2C_PB8_FMW { w: self }
    }
    #[doc = "Bit 19 - Fast Mode Plus (FM+) driving capability activation bits."]
    #[inline]
    pub fn i2c_pb9_fm(&mut self) -> _I2C_PB9_FMW {
        _I2C_PB9_FMW { w: self }
    }
    #[doc = "Bit 20 - I2C1 Fast Mode Plus"]
    #[inline]
    pub fn i2c1_fm(&mut self) -> _I2C1_FMW {
        _I2C1_FMW { w: self }
    }
    #[doc = "Bit 21 - I2C2 Fast Mode Plus"]
    #[inline]
    pub fn i2c2_fm(&mut self) -> _I2C2_FMW {
        _I2C2_FMW { w: self }
    }
    #[doc = "Bits 22:23 - Encoder mode"]
    #[inline]
    pub fn encoder_mode(&mut self) -> _ENCODER_MODEW {
        _ENCODER_MODEW { w: self }
    }
    #[doc = "Bits 26:31 - Interrupt enable bits from FPU"]
    #[inline]
    pub fn fpu_it(&mut self) -> _FPU_ITW {
        _FPU_ITW { w: self }
    }
}
