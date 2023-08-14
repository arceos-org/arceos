#[doc = r" Value read from the register"]
pub struct R {
    bits: u32,
}
#[doc = r" Value to write to the register"]
pub struct W {
    bits: u32,
}
impl super::MODER {
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
#[doc = "Possible values of the field `MODER15`"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MODER15R {
    #[doc = "Input mode (reset state)"]
    INPUT,
    #[doc = "General purpose output mode"]
    OUTPUT,
    #[doc = "Alternate function mode"]
    ALTERNATE,
    #[doc = "Analog mode"]
    ANALOG,
}
impl MODER15R {
    #[doc = r" Value of the field as raw bits"]
    #[inline]
    pub fn bits(&self) -> u8 {
        match *self {
            MODER15R::INPUT => 0,
            MODER15R::OUTPUT => 1,
            MODER15R::ALTERNATE => 2,
            MODER15R::ANALOG => 3,
        }
    }
    #[allow(missing_docs)]
    #[doc(hidden)]
    #[inline]
    pub fn _from(value: u8) -> MODER15R {
        match value {
            0 => MODER15R::INPUT,
            1 => MODER15R::OUTPUT,
            2 => MODER15R::ALTERNATE,
            3 => MODER15R::ANALOG,
            _ => unreachable!(),
        }
    }
    #[doc = "Checks if the value of the field is `INPUT`"]
    #[inline]
    pub fn is_input(&self) -> bool {
        *self == MODER15R::INPUT
    }
    #[doc = "Checks if the value of the field is `OUTPUT`"]
    #[inline]
    pub fn is_output(&self) -> bool {
        *self == MODER15R::OUTPUT
    }
    #[doc = "Checks if the value of the field is `ALTERNATE`"]
    #[inline]
    pub fn is_alternate(&self) -> bool {
        *self == MODER15R::ALTERNATE
    }
    #[doc = "Checks if the value of the field is `ANALOG`"]
    #[inline]
    pub fn is_analog(&self) -> bool {
        *self == MODER15R::ANALOG
    }
}
#[doc = "Possible values of the field `MODER14`"]
pub type MODER14R = MODER15R;
#[doc = "Possible values of the field `MODER13`"]
pub type MODER13R = MODER15R;
#[doc = "Possible values of the field `MODER12`"]
pub type MODER12R = MODER15R;
#[doc = "Possible values of the field `MODER11`"]
pub type MODER11R = MODER15R;
#[doc = "Possible values of the field `MODER10`"]
pub type MODER10R = MODER15R;
#[doc = "Possible values of the field `MODER9`"]
pub type MODER9R = MODER15R;
#[doc = "Possible values of the field `MODER8`"]
pub type MODER8R = MODER15R;
#[doc = "Possible values of the field `MODER7`"]
pub type MODER7R = MODER15R;
#[doc = "Possible values of the field `MODER6`"]
pub type MODER6R = MODER15R;
#[doc = "Possible values of the field `MODER5`"]
pub type MODER5R = MODER15R;
#[doc = "Possible values of the field `MODER4`"]
pub type MODER4R = MODER15R;
#[doc = "Possible values of the field `MODER3`"]
pub type MODER3R = MODER15R;
#[doc = "Possible values of the field `MODER2`"]
pub type MODER2R = MODER15R;
#[doc = "Possible values of the field `MODER1`"]
pub type MODER1R = MODER15R;
#[doc = "Possible values of the field `MODER0`"]
pub type MODER0R = MODER15R;
#[doc = "Values that can be written to the field `MODER15`"]
pub enum MODER15W {
    #[doc = "Input mode (reset state)"]
    INPUT,
    #[doc = "General purpose output mode"]
    OUTPUT,
    #[doc = "Alternate function mode"]
    ALTERNATE,
    #[doc = "Analog mode"]
    ANALOG,
}
impl MODER15W {
    #[allow(missing_docs)]
    #[doc(hidden)]
    #[inline]
    pub fn _bits(&self) -> u8 {
        match *self {
            MODER15W::INPUT => 0,
            MODER15W::OUTPUT => 1,
            MODER15W::ALTERNATE => 2,
            MODER15W::ANALOG => 3,
        }
    }
}
#[doc = r" Proxy"]
pub struct _MODER15W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER15W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER15W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 30;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER14`"]
pub type MODER14W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER14W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER14W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER14W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 28;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER13`"]
pub type MODER13W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER13W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER13W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER13W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 26;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER12`"]
pub type MODER12W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER12W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER12W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER12W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 24;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER11`"]
pub type MODER11W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER11W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER11W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER11W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 22;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER10`"]
pub type MODER10W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER10W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER10W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER10W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 20;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER9`"]
pub type MODER9W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER9W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER9W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER9W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 18;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER8`"]
pub type MODER8W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER8W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER8W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER8W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 16;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER7`"]
pub type MODER7W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER7W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER7W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER7W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 14;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER6`"]
pub type MODER6W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER6W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER6W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER6W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 12;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER5`"]
pub type MODER5W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER5W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER5W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER5W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 10;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER4`"]
pub type MODER4W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER4W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER4W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER4W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 8;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER3`"]
pub type MODER3W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER3W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER3W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER3W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 6;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER2`"]
pub type MODER2W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER2W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER2W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER2W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 4;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER1`"]
pub type MODER1W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER1W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER1W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER1W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
        const MASK: u8 = 3;
        const OFFSET: u8 = 2;
        self.w.bits &= !((MASK as u32) << OFFSET);
        self.w.bits |= ((value & MASK) as u32) << OFFSET;
        self.w
    }
}
#[doc = "Values that can be written to the field `MODER0`"]
pub type MODER0W = MODER15W;
#[doc = r" Proxy"]
pub struct _MODER0W<'a> {
    w: &'a mut W,
}
impl<'a> _MODER0W<'a> {
    #[doc = r" Writes `variant` to the field"]
    #[inline]
    pub fn variant(self, variant: MODER0W) -> &'a mut W {
        {
            self.bits(variant._bits())
        }
    }
    #[doc = "Input mode (reset state)"]
    #[inline]
    pub fn input(self) -> &'a mut W {
        self.variant(MODER15W::INPUT)
    }
    #[doc = "General purpose output mode"]
    #[inline]
    pub fn output(self) -> &'a mut W {
        self.variant(MODER15W::OUTPUT)
    }
    #[doc = "Alternate function mode"]
    #[inline]
    pub fn alternate(self) -> &'a mut W {
        self.variant(MODER15W::ALTERNATE)
    }
    #[doc = "Analog mode"]
    #[inline]
    pub fn analog(self) -> &'a mut W {
        self.variant(MODER15W::ANALOG)
    }
    #[doc = r" Writes raw bits to the field"]
    #[inline]
    pub fn bits(self, value: u8) -> &'a mut W {
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
    #[doc = "Bits 30:31 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder15(&self) -> MODER15R {
        MODER15R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 30;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 28:29 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder14(&self) -> MODER14R {
        MODER14R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 28;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 26:27 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder13(&self) -> MODER13R {
        MODER13R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 26;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 24:25 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder12(&self) -> MODER12R {
        MODER12R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 24;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 22:23 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder11(&self) -> MODER11R {
        MODER11R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 22;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 20:21 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder10(&self) -> MODER10R {
        MODER10R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 20;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 18:19 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder9(&self) -> MODER9R {
        MODER9R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 18;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 16:17 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder8(&self) -> MODER8R {
        MODER8R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 16;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 14:15 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder7(&self) -> MODER7R {
        MODER7R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 14;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 12:13 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder6(&self) -> MODER6R {
        MODER6R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 12;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 10:11 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder5(&self) -> MODER5R {
        MODER5R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 10;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 8:9 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder4(&self) -> MODER4R {
        MODER4R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 8;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 6:7 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder3(&self) -> MODER3R {
        MODER3R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 6;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 4:5 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder2(&self) -> MODER2R {
        MODER2R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 4;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 2:3 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder1(&self) -> MODER1R {
        MODER1R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 2;
            ((self.bits >> OFFSET) & MASK as u32) as u8
        })
    }
    #[doc = "Bits 0:1 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder0(&self) -> MODER0R {
        MODER0R::_from({
            const MASK: u8 = 3;
            const OFFSET: u8 = 0;
            ((self.bits >> OFFSET) & MASK as u32) as u8
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
    #[doc = "Bits 30:31 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder15(&mut self) -> _MODER15W {
        _MODER15W { w: self }
    }
    #[doc = "Bits 28:29 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder14(&mut self) -> _MODER14W {
        _MODER14W { w: self }
    }
    #[doc = "Bits 26:27 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder13(&mut self) -> _MODER13W {
        _MODER13W { w: self }
    }
    #[doc = "Bits 24:25 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder12(&mut self) -> _MODER12W {
        _MODER12W { w: self }
    }
    #[doc = "Bits 22:23 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder11(&mut self) -> _MODER11W {
        _MODER11W { w: self }
    }
    #[doc = "Bits 20:21 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder10(&mut self) -> _MODER10W {
        _MODER10W { w: self }
    }
    #[doc = "Bits 18:19 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder9(&mut self) -> _MODER9W {
        _MODER9W { w: self }
    }
    #[doc = "Bits 16:17 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder8(&mut self) -> _MODER8W {
        _MODER8W { w: self }
    }
    #[doc = "Bits 14:15 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder7(&mut self) -> _MODER7W {
        _MODER7W { w: self }
    }
    #[doc = "Bits 12:13 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder6(&mut self) -> _MODER6W {
        _MODER6W { w: self }
    }
    #[doc = "Bits 10:11 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder5(&mut self) -> _MODER5W {
        _MODER5W { w: self }
    }
    #[doc = "Bits 8:9 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder4(&mut self) -> _MODER4W {
        _MODER4W { w: self }
    }
    #[doc = "Bits 6:7 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder3(&mut self) -> _MODER3W {
        _MODER3W { w: self }
    }
    #[doc = "Bits 4:5 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder2(&mut self) -> _MODER2W {
        _MODER2W { w: self }
    }
    #[doc = "Bits 2:3 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder1(&mut self) -> _MODER1W {
        _MODER1W { w: self }
    }
    #[doc = "Bits 0:1 - Port x configuration bits (y = 0..15)"]
    #[inline]
    pub fn moder0(&mut self) -> _MODER0W {
        _MODER0W { w: self }
    }
}
