#![doc = "Peripheral access API for STM32F30X microcontrollers (generated using svd2rust v0.14.0)\n\nYou can find an overview of the API [here].\n\n[here]: https://docs.rs/svd2rust/0.14.0/svd2rust/#peripheral-api"]
#![deny(missing_docs)]
#![deny(warnings)]
#![allow(non_camel_case_types)]
#![no_std]
extern crate bare_metal;
extern crate cortex_m;
#[cfg(feature = "rt")]
extern crate cortex_m_rt;
extern crate vcell;
use core::marker::PhantomData;
use core::ops::Deref;
#[doc = r" Number available in the NVIC for configuring priority"]
pub const NVIC_PRIO_BITS: u8 = 4;
#[cfg(feature = "rt")]
extern "C" {
    fn WWDG();
    fn PVD();
    fn TAMP_STAMP();
    fn RTC_WKUP();
    fn FLASH();
    fn RCC();
    fn EXTI0();
    fn EXTI1();
    fn EXTI2_TSC();
    fn EXTI3();
    fn EXTI4();
    fn DMA1_CH1();
    fn DMA1_CH2();
    fn DMA1_CH3();
    fn DMA1_CH4();
    fn DMA1_CH5();
    fn DMA1_CH6();
    fn DMA1_CH7();
    fn ADC1_2();
    fn USB_HP_CAN_TX();
    fn USB_LP_CAN_RX0();
    fn CAN_RX1();
    fn CAN_SCE();
    fn EXTI9_5();
    fn TIM1_BRK_TIM15();
    fn TIM1_UP_TIM16();
    fn TIM1_TRG_COM_TIM17();
    fn TIM1_CC();
    fn TIM2();
    fn TIM3();
    fn TIM4();
    fn I2C1_EV_EXTI23();
    fn I2C1_ER();
    fn I2C2_EV_EXTI24();
    fn I2C2_ER();
    fn SPI1();
    fn SPI2();
    fn USART1_EXTI25();
    fn USART2_EXTI26();
    fn USART3_EXTI28();
    fn EXTI15_10();
    fn RTCALARM();
    fn USB_WKUP();
    fn TIM8_BRK();
    fn TIM8_UP();
    fn TIM8_TRG_COM();
    fn TIM8_CC();
    fn ADC3();
    fn SPI3();
    fn UART4_EXTI34();
    fn UART5_EXTI35();
    fn TIM6_DACUNDER();
    fn TIM7();
    fn DMA2_CH1();
    fn DMA2_CH2();
    fn DMA2_CH3();
    fn DMA2_CH4();
    fn DMA2_CH5();
    fn ADC4();
    fn COMP123();
    fn COMP456();
    fn COMP7();
    fn USB_HP();
    fn USB_LP();
    fn USB_WKUP_EXTI();
    fn FPU();
}
#[doc(hidden)]
pub union Vector {
    _handler: unsafe extern "C" fn(),
    _reserved: u32,
}
#[cfg(feature = "rt")]
#[doc(hidden)]
#[link_section = ".vector_table.interrupts"]
#[no_mangle]
pub static __INTERRUPTS: [Vector; 82] = [
    Vector { _handler: WWDG },
    Vector { _handler: PVD },
    Vector {
        _handler: TAMP_STAMP,
    },
    Vector { _handler: RTC_WKUP },
    Vector { _handler: FLASH },
    Vector { _handler: RCC },
    Vector { _handler: EXTI0 },
    Vector { _handler: EXTI1 },
    Vector {
        _handler: EXTI2_TSC,
    },
    Vector { _handler: EXTI3 },
    Vector { _handler: EXTI4 },
    Vector { _handler: DMA1_CH1 },
    Vector { _handler: DMA1_CH2 },
    Vector { _handler: DMA1_CH3 },
    Vector { _handler: DMA1_CH4 },
    Vector { _handler: DMA1_CH5 },
    Vector { _handler: DMA1_CH6 },
    Vector { _handler: DMA1_CH7 },
    Vector { _handler: ADC1_2 },
    Vector {
        _handler: USB_HP_CAN_TX,
    },
    Vector {
        _handler: USB_LP_CAN_RX0,
    },
    Vector { _handler: CAN_RX1 },
    Vector { _handler: CAN_SCE },
    Vector { _handler: EXTI9_5 },
    Vector {
        _handler: TIM1_BRK_TIM15,
    },
    Vector {
        _handler: TIM1_UP_TIM16,
    },
    Vector {
        _handler: TIM1_TRG_COM_TIM17,
    },
    Vector { _handler: TIM1_CC },
    Vector { _handler: TIM2 },
    Vector { _handler: TIM3 },
    Vector { _handler: TIM4 },
    Vector {
        _handler: I2C1_EV_EXTI23,
    },
    Vector { _handler: I2C1_ER },
    Vector {
        _handler: I2C2_EV_EXTI24,
    },
    Vector { _handler: I2C2_ER },
    Vector { _handler: SPI1 },
    Vector { _handler: SPI2 },
    Vector {
        _handler: USART1_EXTI25,
    },
    Vector {
        _handler: USART2_EXTI26,
    },
    Vector {
        _handler: USART3_EXTI28,
    },
    Vector {
        _handler: EXTI15_10,
    },
    Vector { _handler: RTCALARM },
    Vector { _handler: USB_WKUP },
    Vector { _handler: TIM8_BRK },
    Vector { _handler: TIM8_UP },
    Vector {
        _handler: TIM8_TRG_COM,
    },
    Vector { _handler: TIM8_CC },
    Vector { _handler: ADC3 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _handler: SPI3 },
    Vector {
        _handler: UART4_EXTI34,
    },
    Vector {
        _handler: UART5_EXTI35,
    },
    Vector {
        _handler: TIM6_DACUNDER,
    },
    Vector { _handler: TIM7 },
    Vector { _handler: DMA2_CH1 },
    Vector { _handler: DMA2_CH2 },
    Vector { _handler: DMA2_CH3 },
    Vector { _handler: DMA2_CH4 },
    Vector { _handler: DMA2_CH5 },
    Vector { _handler: ADC4 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _handler: COMP123 },
    Vector { _handler: COMP456 },
    Vector { _handler: COMP7 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _handler: USB_HP },
    Vector { _handler: USB_LP },
    Vector {
        _handler: USB_WKUP_EXTI,
    },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _handler: FPU },
];
#[doc = r" Enumeration of all the interrupts"]
pub enum Interrupt {
    #[doc = "0 - Window Watchdog interrupt"]
    WWDG,
    #[doc = "1 - PVD through EXTI line detection interrupt"]
    PVD,
    #[doc = "2 - Tamper and TimeStamp interrupts"]
    TAMP_STAMP,
    #[doc = "3 - RTC Wakeup interrupt through the EXTI line"]
    RTC_WKUP,
    #[doc = "4 - Flash global interrupt"]
    FLASH,
    #[doc = "5 - RCC global interrupt"]
    RCC,
    #[doc = "6 - EXTI Line0 interrupt"]
    EXTI0,
    #[doc = "7 - EXTI Line3 interrupt"]
    EXTI1,
    #[doc = "8 - EXTI Line2 and Touch sensing interrupts"]
    EXTI2_TSC,
    #[doc = "9 - EXTI Line3 interrupt"]
    EXTI3,
    #[doc = "10 - EXTI Line4 interrupt"]
    EXTI4,
    #[doc = "11 - DMA1 channel 1 interrupt"]
    DMA1_CH1,
    #[doc = "12 - DMA1 channel 2 interrupt"]
    DMA1_CH2,
    #[doc = "13 - DMA1 channel 3 interrupt"]
    DMA1_CH3,
    #[doc = "14 - DMA1 channel 4 interrupt"]
    DMA1_CH4,
    #[doc = "15 - DMA1 channel 5 interrupt"]
    DMA1_CH5,
    #[doc = "16 - DMA1 channel 6 interrupt"]
    DMA1_CH6,
    #[doc = "17 - DMA1 channel 7interrupt"]
    DMA1_CH7,
    #[doc = "18 - ADC1 and ADC2 global interrupt"]
    ADC1_2,
    #[doc = "19 - USB High Priority/CAN_TX interrupts"]
    USB_HP_CAN_TX,
    #[doc = "20 - USB Low Priority/CAN_RX0 interrupts"]
    USB_LP_CAN_RX0,
    #[doc = "21 - CAN_RX1 interrupt"]
    CAN_RX1,
    #[doc = "22 - CAN_SCE interrupt"]
    CAN_SCE,
    #[doc = "23 - EXTI Line5 to Line9 interrupts"]
    EXTI9_5,
    #[doc = "24 - TIM1 Break/TIM15 global interruts"]
    TIM1_BRK_TIM15,
    #[doc = "25 - TIM1 Update/TIM16 global interrupts"]
    TIM1_UP_TIM16,
    #[doc = "26 - TIM1 trigger and commutation/TIM17 interrupts"]
    TIM1_TRG_COM_TIM17,
    #[doc = "27 - TIM1 capture compare interrupt"]
    TIM1_CC,
    #[doc = "28 - TIM2 global interrupt"]
    TIM2,
    #[doc = "29 - TIM3 global interrupt"]
    TIM3,
    #[doc = "30 - TIM4 global interrupt"]
    TIM4,
    #[doc = "31 - I2C1 event interrupt and EXTI Line23 interrupt"]
    I2C1_EV_EXTI23,
    #[doc = "32 - I2C1 error interrupt"]
    I2C1_ER,
    #[doc = "33 - I2C2 event interrupt & EXTI Line24 interrupt"]
    I2C2_EV_EXTI24,
    #[doc = "34 - I2C2 error interrupt"]
    I2C2_ER,
    #[doc = "35 - SPI1 global interrupt"]
    SPI1,
    #[doc = "36 - SPI2 global interrupt"]
    SPI2,
    #[doc = "37 - USART1 global interrupt and EXTI Line 25 interrupt"]
    USART1_EXTI25,
    #[doc = "38 - USART2 global interrupt and EXTI Line 26 interrupt"]
    USART2_EXTI26,
    #[doc = "39 - USART3 global interrupt and EXTI Line 28 interrupt"]
    USART3_EXTI28,
    #[doc = "40 - EXTI Line15 to Line10 interrupts"]
    EXTI15_10,
    #[doc = "41 - RTC alarm interrupt"]
    RTCALARM,
    #[doc = "42 - USB wakeup from Suspend"]
    USB_WKUP,
    #[doc = "43 - TIM8 break interrupt"]
    TIM8_BRK,
    #[doc = "44 - TIM8 update interrupt"]
    TIM8_UP,
    #[doc = "45 - TIM8 Trigger and commutation interrupts"]
    TIM8_TRG_COM,
    #[doc = "46 - TIM8 capture compare interrupt"]
    TIM8_CC,
    #[doc = "47 - ADC3 global interrupt"]
    ADC3,
    #[doc = "51 - SPI3 global interrupt"]
    SPI3,
    #[doc = "52 - UART4 global and EXTI Line 34 interrupts"]
    UART4_EXTI34,
    #[doc = "53 - UART5 global and EXTI Line 35 interrupts"]
    UART5_EXTI35,
    #[doc = "54 - TIM6 global and DAC12 underrun interrupts"]
    TIM6_DACUNDER,
    #[doc = "55 - TIM7 global interrupt"]
    TIM7,
    #[doc = "56 - DMA2 channel1 global interrupt"]
    DMA2_CH1,
    #[doc = "57 - DMA2 channel2 global interrupt"]
    DMA2_CH2,
    #[doc = "58 - DMA2 channel3 global interrupt"]
    DMA2_CH3,
    #[doc = "59 - DMA2 channel4 global interrupt"]
    DMA2_CH4,
    #[doc = "60 - DMA2 channel5 global interrupt"]
    DMA2_CH5,
    #[doc = "61 - ADC4 global interrupt"]
    ADC4,
    #[doc = "64 - COMP1 & COMP2 & COMP3 interrupts combined with EXTI Lines 21, 22 and 29 interrupts"]
    COMP123,
    #[doc = "65 - COMP4 & COMP5 & COMP6 interrupts combined with EXTI Lines 30, 31 and 32 interrupts"]
    COMP456,
    #[doc = "66 - COMP7 interrupt combined with EXTI Line 33 interrupt"]
    COMP7,
    #[doc = "74 - USB High priority interrupt"]
    USB_HP,
    #[doc = "75 - USB Low priority interrupt"]
    USB_LP,
    #[doc = "76 - USB wakeup from Suspend and EXTI Line 18"]
    USB_WKUP_EXTI,
    #[doc = "81 - Floating point interrupt"]
    FPU,
}
unsafe impl ::bare_metal::Nr for Interrupt {
    #[inline]
    fn nr(&self) -> u8 {
        match *self {
            Interrupt::WWDG => 0,
            Interrupt::PVD => 1,
            Interrupt::TAMP_STAMP => 2,
            Interrupt::RTC_WKUP => 3,
            Interrupt::FLASH => 4,
            Interrupt::RCC => 5,
            Interrupt::EXTI0 => 6,
            Interrupt::EXTI1 => 7,
            Interrupt::EXTI2_TSC => 8,
            Interrupt::EXTI3 => 9,
            Interrupt::EXTI4 => 10,
            Interrupt::DMA1_CH1 => 11,
            Interrupt::DMA1_CH2 => 12,
            Interrupt::DMA1_CH3 => 13,
            Interrupt::DMA1_CH4 => 14,
            Interrupt::DMA1_CH5 => 15,
            Interrupt::DMA1_CH6 => 16,
            Interrupt::DMA1_CH7 => 17,
            Interrupt::ADC1_2 => 18,
            Interrupt::USB_HP_CAN_TX => 19,
            Interrupt::USB_LP_CAN_RX0 => 20,
            Interrupt::CAN_RX1 => 21,
            Interrupt::CAN_SCE => 22,
            Interrupt::EXTI9_5 => 23,
            Interrupt::TIM1_BRK_TIM15 => 24,
            Interrupt::TIM1_UP_TIM16 => 25,
            Interrupt::TIM1_TRG_COM_TIM17 => 26,
            Interrupt::TIM1_CC => 27,
            Interrupt::TIM2 => 28,
            Interrupt::TIM3 => 29,
            Interrupt::TIM4 => 30,
            Interrupt::I2C1_EV_EXTI23 => 31,
            Interrupt::I2C1_ER => 32,
            Interrupt::I2C2_EV_EXTI24 => 33,
            Interrupt::I2C2_ER => 34,
            Interrupt::SPI1 => 35,
            Interrupt::SPI2 => 36,
            Interrupt::USART1_EXTI25 => 37,
            Interrupt::USART2_EXTI26 => 38,
            Interrupt::USART3_EXTI28 => 39,
            Interrupt::EXTI15_10 => 40,
            Interrupt::RTCALARM => 41,
            Interrupt::USB_WKUP => 42,
            Interrupt::TIM8_BRK => 43,
            Interrupt::TIM8_UP => 44,
            Interrupt::TIM8_TRG_COM => 45,
            Interrupt::TIM8_CC => 46,
            Interrupt::ADC3 => 47,
            Interrupt::SPI3 => 51,
            Interrupt::UART4_EXTI34 => 52,
            Interrupt::UART5_EXTI35 => 53,
            Interrupt::TIM6_DACUNDER => 54,
            Interrupt::TIM7 => 55,
            Interrupt::DMA2_CH1 => 56,
            Interrupt::DMA2_CH2 => 57,
            Interrupt::DMA2_CH3 => 58,
            Interrupt::DMA2_CH4 => 59,
            Interrupt::DMA2_CH5 => 60,
            Interrupt::ADC4 => 61,
            Interrupt::COMP123 => 64,
            Interrupt::COMP456 => 65,
            Interrupt::COMP7 => 66,
            Interrupt::USB_HP => 74,
            Interrupt::USB_LP => 75,
            Interrupt::USB_WKUP_EXTI => 76,
            Interrupt::FPU => 81,
        }
    }
}
#[cfg(feature = "rt")]
pub use self::Interrupt as interrupt;
pub use cortex_m::peripheral::Peripherals as CorePeripherals;
pub use cortex_m::peripheral::{CBP, CPUID, DCB, DWT, FPB, FPU, ITM, MPU, NVIC, SCB, SYST, TPIU};
#[cfg(feature = "rt")]
pub use cortex_m_rt::interrupt;
#[doc = "General-purpose I/Os"]
pub struct GPIOA {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for GPIOA {}
impl GPIOA {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const gpioa::RegisterBlock {
        1207959552 as *const _
    }
}
impl Deref for GPIOA {
    type Target = gpioa::RegisterBlock;
    fn deref(&self) -> &gpioa::RegisterBlock {
        unsafe { &*GPIOA::ptr() }
    }
}
#[doc = "General-purpose I/Os"]
pub mod gpioa;
#[doc = "General-purpose I/Os"]
pub struct GPIOB {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for GPIOB {}
impl GPIOB {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const gpiob::RegisterBlock {
        1207960576 as *const _
    }
}
impl Deref for GPIOB {
    type Target = gpiob::RegisterBlock;
    fn deref(&self) -> &gpiob::RegisterBlock {
        unsafe { &*GPIOB::ptr() }
    }
}
#[doc = "General-purpose I/Os"]
pub mod gpiob;
#[doc = "General-purpose I/Os"]
pub struct GPIOC {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for GPIOC {}
impl GPIOC {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const gpioc::RegisterBlock {
        1207961600 as *const _
    }
}
impl Deref for GPIOC {
    type Target = gpioc::RegisterBlock;
    fn deref(&self) -> &gpioc::RegisterBlock {
        unsafe { &*GPIOC::ptr() }
    }
}
#[doc = "General-purpose I/Os"]
pub mod gpioc;
#[doc = "GPIOD"]
pub struct GPIOD {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for GPIOD {}
impl GPIOD {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const gpioc::RegisterBlock {
        1207962624 as *const _
    }
}
impl Deref for GPIOD {
    type Target = gpioc::RegisterBlock;
    fn deref(&self) -> &gpioc::RegisterBlock {
        unsafe { &*GPIOD::ptr() }
    }
}
#[doc = "GPIOE"]
pub struct GPIOE {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for GPIOE {}
impl GPIOE {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const gpioc::RegisterBlock {
        1207963648 as *const _
    }
}
impl Deref for GPIOE {
    type Target = gpioc::RegisterBlock;
    fn deref(&self) -> &gpioc::RegisterBlock {
        unsafe { &*GPIOE::ptr() }
    }
}
#[doc = "GPIOF"]
pub struct GPIOF {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for GPIOF {}
impl GPIOF {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const gpioc::RegisterBlock {
        1207964672 as *const _
    }
}
impl Deref for GPIOF {
    type Target = gpioc::RegisterBlock;
    fn deref(&self) -> &gpioc::RegisterBlock {
        unsafe { &*GPIOF::ptr() }
    }
}
#[doc = "Touch sensing controller"]
pub struct TSC {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TSC {}
impl TSC {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const tsc::RegisterBlock {
        1073889280 as *const _
    }
}
impl Deref for TSC {
    type Target = tsc::RegisterBlock;
    fn deref(&self) -> &tsc::RegisterBlock {
        unsafe { &*TSC::ptr() }
    }
}
#[doc = "Touch sensing controller"]
pub mod tsc;
#[doc = "cyclic redundancy check calculation unit"]
pub struct CRC {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for CRC {}
impl CRC {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const crc::RegisterBlock {
        1073885184 as *const _
    }
}
impl Deref for CRC {
    type Target = crc::RegisterBlock;
    fn deref(&self) -> &crc::RegisterBlock {
        unsafe { &*CRC::ptr() }
    }
}
#[doc = "cyclic redundancy check calculation unit"]
pub mod crc;
#[doc = "Flash"]
pub struct FLASH {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for FLASH {}
impl FLASH {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const flash::RegisterBlock {
        1073881088 as *const _
    }
}
impl Deref for FLASH {
    type Target = flash::RegisterBlock;
    fn deref(&self) -> &flash::RegisterBlock {
        unsafe { &*FLASH::ptr() }
    }
}
#[doc = "Flash"]
pub mod flash;
#[doc = "Reset and clock control"]
pub struct RCC {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for RCC {}
impl RCC {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const rcc::RegisterBlock {
        1073876992 as *const _
    }
}
impl Deref for RCC {
    type Target = rcc::RegisterBlock;
    fn deref(&self) -> &rcc::RegisterBlock {
        unsafe { &*RCC::ptr() }
    }
}
#[doc = "Reset and clock control"]
pub mod rcc;
#[doc = "DMA controller 1"]
pub struct DMA1 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for DMA1 {}
impl DMA1 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const dma1::RegisterBlock {
        1073872896 as *const _
    }
}
impl Deref for DMA1 {
    type Target = dma1::RegisterBlock;
    fn deref(&self) -> &dma1::RegisterBlock {
        unsafe { &*DMA1::ptr() }
    }
}
#[doc = "DMA controller 1"]
pub mod dma1;
#[doc = "DMA2"]
pub struct DMA2 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for DMA2 {}
impl DMA2 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const dma1::RegisterBlock {
        1073873920 as *const _
    }
}
impl Deref for DMA2 {
    type Target = dma1::RegisterBlock;
    fn deref(&self) -> &dma1::RegisterBlock {
        unsafe { &*DMA2::ptr() }
    }
}
#[doc = "General purpose timer"]
pub struct TIM2 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TIM2 {}
impl TIM2 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const tim2::RegisterBlock {
        1073741824 as *const _
    }
}
impl Deref for TIM2 {
    type Target = tim2::RegisterBlock;
    fn deref(&self) -> &tim2::RegisterBlock {
        unsafe { &*TIM2::ptr() }
    }
}
#[doc = "General purpose timer"]
pub mod tim2;
#[doc = "TIM3"]
pub struct TIM3 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TIM3 {}
impl TIM3 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const tim2::RegisterBlock {
        1073742848 as *const _
    }
}
impl Deref for TIM3 {
    type Target = tim2::RegisterBlock;
    fn deref(&self) -> &tim2::RegisterBlock {
        unsafe { &*TIM3::ptr() }
    }
}
#[doc = "TIM4"]
pub struct TIM4 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TIM4 {}
impl TIM4 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const tim2::RegisterBlock {
        1073743872 as *const _
    }
}
impl Deref for TIM4 {
    type Target = tim2::RegisterBlock;
    fn deref(&self) -> &tim2::RegisterBlock {
        unsafe { &*TIM4::ptr() }
    }
}
#[doc = "General purpose timers"]
pub struct TIM15 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TIM15 {}
impl TIM15 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const tim15::RegisterBlock {
        1073823744 as *const _
    }
}
impl Deref for TIM15 {
    type Target = tim15::RegisterBlock;
    fn deref(&self) -> &tim15::RegisterBlock {
        unsafe { &*TIM15::ptr() }
    }
}
#[doc = "General purpose timers"]
pub mod tim15;
#[doc = "General-purpose-timers"]
pub struct TIM16 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TIM16 {}
impl TIM16 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const tim16::RegisterBlock {
        1073824768 as *const _
    }
}
impl Deref for TIM16 {
    type Target = tim16::RegisterBlock;
    fn deref(&self) -> &tim16::RegisterBlock {
        unsafe { &*TIM16::ptr() }
    }
}
#[doc = "General-purpose-timers"]
pub mod tim16;
#[doc = "General purpose timer"]
pub struct TIM17 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TIM17 {}
impl TIM17 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const tim17::RegisterBlock {
        1073825792 as *const _
    }
}
impl Deref for TIM17 {
    type Target = tim17::RegisterBlock;
    fn deref(&self) -> &tim17::RegisterBlock {
        unsafe { &*TIM17::ptr() }
    }
}
#[doc = "General purpose timer"]
pub mod tim17;
#[doc = "Universal synchronous asynchronous receiver transmitter"]
pub struct USART1 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for USART1 {}
impl USART1 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const usart1::RegisterBlock {
        1073821696 as *const _
    }
}
impl Deref for USART1 {
    type Target = usart1::RegisterBlock;
    fn deref(&self) -> &usart1::RegisterBlock {
        unsafe { &*USART1::ptr() }
    }
}
#[doc = "Universal synchronous asynchronous receiver transmitter"]
pub mod usart1;
#[doc = "USART2"]
pub struct USART2 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for USART2 {}
impl USART2 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const usart1::RegisterBlock {
        1073759232 as *const _
    }
}
impl Deref for USART2 {
    type Target = usart1::RegisterBlock;
    fn deref(&self) -> &usart1::RegisterBlock {
        unsafe { &*USART2::ptr() }
    }
}
#[doc = "USART3"]
pub struct USART3 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for USART3 {}
impl USART3 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const usart1::RegisterBlock {
        1073760256 as *const _
    }
}
impl Deref for USART3 {
    type Target = usart1::RegisterBlock;
    fn deref(&self) -> &usart1::RegisterBlock {
        unsafe { &*USART3::ptr() }
    }
}
#[doc = "UART4"]
pub struct UART4 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for UART4 {}
impl UART4 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const usart1::RegisterBlock {
        1073761280 as *const _
    }
}
impl Deref for UART4 {
    type Target = usart1::RegisterBlock;
    fn deref(&self) -> &usart1::RegisterBlock {
        unsafe { &*UART4::ptr() }
    }
}
#[doc = "UART5"]
pub struct UART5 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for UART5 {}
impl UART5 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const usart1::RegisterBlock {
        1073762304 as *const _
    }
}
impl Deref for UART5 {
    type Target = usart1::RegisterBlock;
    fn deref(&self) -> &usart1::RegisterBlock {
        unsafe { &*UART5::ptr() }
    }
}
#[doc = "Serial peripheral interface/Inter-IC sound"]
pub struct SPI1 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for SPI1 {}
impl SPI1 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const spi1::RegisterBlock {
        1073819648 as *const _
    }
}
impl Deref for SPI1 {
    type Target = spi1::RegisterBlock;
    fn deref(&self) -> &spi1::RegisterBlock {
        unsafe { &*SPI1::ptr() }
    }
}
#[doc = "Serial peripheral interface/Inter-IC sound"]
pub mod spi1;
#[doc = "SPI2"]
pub struct SPI2 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for SPI2 {}
impl SPI2 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const spi1::RegisterBlock {
        1073756160 as *const _
    }
}
impl Deref for SPI2 {
    type Target = spi1::RegisterBlock;
    fn deref(&self) -> &spi1::RegisterBlock {
        unsafe { &*SPI2::ptr() }
    }
}
#[doc = "SPI3"]
pub struct SPI3 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for SPI3 {}
impl SPI3 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const spi1::RegisterBlock {
        1073757184 as *const _
    }
}
impl Deref for SPI3 {
    type Target = spi1::RegisterBlock;
    fn deref(&self) -> &spi1::RegisterBlock {
        unsafe { &*SPI3::ptr() }
    }
}
#[doc = "I2S2ext"]
pub struct I2S2EXT {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for I2S2EXT {}
impl I2S2EXT {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const spi1::RegisterBlock {
        1073755136 as *const _
    }
}
impl Deref for I2S2EXT {
    type Target = spi1::RegisterBlock;
    fn deref(&self) -> &spi1::RegisterBlock {
        unsafe { &*I2S2EXT::ptr() }
    }
}
#[doc = "I2S3ext"]
pub struct I2S3EXT {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for I2S3EXT {}
impl I2S3EXT {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const spi1::RegisterBlock {
        1073758208 as *const _
    }
}
impl Deref for I2S3EXT {
    type Target = spi1::RegisterBlock;
    fn deref(&self) -> &spi1::RegisterBlock {
        unsafe { &*I2S3EXT::ptr() }
    }
}
#[doc = "External interrupt/event controller"]
pub struct EXTI {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for EXTI {}
impl EXTI {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const exti::RegisterBlock {
        1073808384 as *const _
    }
}
impl Deref for EXTI {
    type Target = exti::RegisterBlock;
    fn deref(&self) -> &exti::RegisterBlock {
        unsafe { &*EXTI::ptr() }
    }
}
#[doc = "External interrupt/event controller"]
pub mod exti;
#[doc = "Comparator"]
pub struct COMP {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for COMP {}
impl COMP {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const comp::RegisterBlock {
        1073807388 as *const _
    }
}
impl Deref for COMP {
    type Target = comp::RegisterBlock;
    fn deref(&self) -> &comp::RegisterBlock {
        unsafe { &*COMP::ptr() }
    }
}
#[doc = "Comparator"]
pub mod comp;
#[doc = "Power control"]
pub struct PWR {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for PWR {}
impl PWR {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const pwr::RegisterBlock {
        1073770496 as *const _
    }
}
impl Deref for PWR {
    type Target = pwr::RegisterBlock;
    fn deref(&self) -> &pwr::RegisterBlock {
        unsafe { &*PWR::ptr() }
    }
}
#[doc = "Power control"]
pub mod pwr;
#[doc = "Controller area network"]
pub struct CAN {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for CAN {}
impl CAN {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const can::RegisterBlock {
        1073767424 as *const _
    }
}
impl Deref for CAN {
    type Target = can::RegisterBlock;
    fn deref(&self) -> &can::RegisterBlock {
        unsafe { &*CAN::ptr() }
    }
}
#[doc = "Controller area network"]
pub mod can;
#[doc = "Universal serial bus full-speed device interface"]
pub struct USB_FS {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for USB_FS {}
impl USB_FS {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const usb_fs::RegisterBlock {
        1073765376 as *const _
    }
}
impl Deref for USB_FS {
    type Target = usb_fs::RegisterBlock;
    fn deref(&self) -> &usb_fs::RegisterBlock {
        unsafe { &*USB_FS::ptr() }
    }
}
#[doc = "Universal serial bus full-speed device interface"]
pub mod usb_fs;
#[doc = "Inter-integrated circuit"]
pub struct I2C1 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for I2C1 {}
impl I2C1 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const i2c1::RegisterBlock {
        1073763328 as *const _
    }
}
impl Deref for I2C1 {
    type Target = i2c1::RegisterBlock;
    fn deref(&self) -> &i2c1::RegisterBlock {
        unsafe { &*I2C1::ptr() }
    }
}
#[doc = "Inter-integrated circuit"]
pub mod i2c1;
#[doc = "I2C2"]
pub struct I2C2 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for I2C2 {}
impl I2C2 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const i2c1::RegisterBlock {
        1073764352 as *const _
    }
}
impl Deref for I2C2 {
    type Target = i2c1::RegisterBlock;
    fn deref(&self) -> &i2c1::RegisterBlock {
        unsafe { &*I2C2::ptr() }
    }
}
#[doc = "Independent watchdog"]
pub struct IWDG {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for IWDG {}
impl IWDG {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const iwdg::RegisterBlock {
        1073754112 as *const _
    }
}
impl Deref for IWDG {
    type Target = iwdg::RegisterBlock;
    fn deref(&self) -> &iwdg::RegisterBlock {
        unsafe { &*IWDG::ptr() }
    }
}
#[doc = "Independent watchdog"]
pub mod iwdg;
#[doc = "Window watchdog"]
pub struct WWDG {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for WWDG {}
impl WWDG {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const wwdg::RegisterBlock {
        1073753088 as *const _
    }
}
impl Deref for WWDG {
    type Target = wwdg::RegisterBlock;
    fn deref(&self) -> &wwdg::RegisterBlock {
        unsafe { &*WWDG::ptr() }
    }
}
#[doc = "Window watchdog"]
pub mod wwdg;
#[doc = "Real-time clock"]
pub struct RTC {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for RTC {}
impl RTC {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const rtc::RegisterBlock {
        1073752064 as *const _
    }
}
impl Deref for RTC {
    type Target = rtc::RegisterBlock;
    fn deref(&self) -> &rtc::RegisterBlock {
        unsafe { &*RTC::ptr() }
    }
}
#[doc = "Real-time clock"]
pub mod rtc;
#[doc = "Basic timers"]
pub struct TIM6 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TIM6 {}
impl TIM6 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const tim6::RegisterBlock {
        1073745920 as *const _
    }
}
impl Deref for TIM6 {
    type Target = tim6::RegisterBlock;
    fn deref(&self) -> &tim6::RegisterBlock {
        unsafe { &*TIM6::ptr() }
    }
}
#[doc = "Basic timers"]
pub mod tim6;
#[doc = "TIM7"]
pub struct TIM7 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TIM7 {}
impl TIM7 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const tim6::RegisterBlock {
        1073746944 as *const _
    }
}
impl Deref for TIM7 {
    type Target = tim6::RegisterBlock;
    fn deref(&self) -> &tim6::RegisterBlock {
        unsafe { &*TIM7::ptr() }
    }
}
#[doc = "Digital-to-analog converter"]
pub struct DAC {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for DAC {}
impl DAC {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const dac::RegisterBlock {
        1073771520 as *const _
    }
}
impl Deref for DAC {
    type Target = dac::RegisterBlock;
    fn deref(&self) -> &dac::RegisterBlock {
        unsafe { &*DAC::ptr() }
    }
}
#[doc = "Digital-to-analog converter"]
pub mod dac;
#[doc = "Debug support"]
pub struct DBGMCU {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for DBGMCU {}
impl DBGMCU {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const dbgmcu::RegisterBlock {
        3758366720 as *const _
    }
}
impl Deref for DBGMCU {
    type Target = dbgmcu::RegisterBlock;
    fn deref(&self) -> &dbgmcu::RegisterBlock {
        unsafe { &*DBGMCU::ptr() }
    }
}
#[doc = "Debug support"]
pub mod dbgmcu;
#[doc = "Advanced timer"]
pub struct TIM1 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TIM1 {}
impl TIM1 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const tim1::RegisterBlock {
        1073818624 as *const _
    }
}
impl Deref for TIM1 {
    type Target = tim1::RegisterBlock;
    fn deref(&self) -> &tim1::RegisterBlock {
        unsafe { &*TIM1::ptr() }
    }
}
#[doc = "Advanced timer"]
pub mod tim1;
#[doc = "Advanced-timers"]
pub struct TIM8 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TIM8 {}
impl TIM8 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const tim8::RegisterBlock {
        1073820672 as *const _
    }
}
impl Deref for TIM8 {
    type Target = tim8::RegisterBlock;
    fn deref(&self) -> &tim8::RegisterBlock {
        unsafe { &*TIM8::ptr() }
    }
}
#[doc = "Advanced-timers"]
pub mod tim8;
#[doc = "Analog-to-Digital Converter"]
pub struct ADC1 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for ADC1 {}
impl ADC1 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const adc1::RegisterBlock {
        1342177280 as *const _
    }
}
impl Deref for ADC1 {
    type Target = adc1::RegisterBlock;
    fn deref(&self) -> &adc1::RegisterBlock {
        unsafe { &*ADC1::ptr() }
    }
}
#[doc = "Analog-to-Digital Converter"]
pub mod adc1;
#[doc = "ADC2"]
pub struct ADC2 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for ADC2 {}
impl ADC2 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const adc1::RegisterBlock {
        1342177536 as *const _
    }
}
impl Deref for ADC2 {
    type Target = adc1::RegisterBlock;
    fn deref(&self) -> &adc1::RegisterBlock {
        unsafe { &*ADC2::ptr() }
    }
}
#[doc = "ADC3"]
pub struct ADC3 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for ADC3 {}
impl ADC3 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const adc1::RegisterBlock {
        1342178304 as *const _
    }
}
impl Deref for ADC3 {
    type Target = adc1::RegisterBlock;
    fn deref(&self) -> &adc1::RegisterBlock {
        unsafe { &*ADC3::ptr() }
    }
}
#[doc = "ADC4"]
pub struct ADC4 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for ADC4 {}
impl ADC4 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const adc1::RegisterBlock {
        1342178560 as *const _
    }
}
impl Deref for ADC4 {
    type Target = adc1::RegisterBlock;
    fn deref(&self) -> &adc1::RegisterBlock {
        unsafe { &*ADC4::ptr() }
    }
}
#[doc = "Analog-to-Digital Converter"]
pub struct ADC1_2 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for ADC1_2 {}
impl ADC1_2 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const adc1_2::RegisterBlock {
        1342178048 as *const _
    }
}
impl Deref for ADC1_2 {
    type Target = adc1_2::RegisterBlock;
    fn deref(&self) -> &adc1_2::RegisterBlock {
        unsafe { &*ADC1_2::ptr() }
    }
}
#[doc = "Analog-to-Digital Converter"]
pub mod adc1_2;
#[doc = "ADC3_4"]
pub struct ADC3_4 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for ADC3_4 {}
impl ADC3_4 {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const adc1_2::RegisterBlock {
        1342179072 as *const _
    }
}
impl Deref for ADC3_4 {
    type Target = adc1_2::RegisterBlock;
    fn deref(&self) -> &adc1_2::RegisterBlock {
        unsafe { &*ADC3_4::ptr() }
    }
}
#[doc = "System configuration controller"]
pub struct SYSCFG {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for SYSCFG {}
impl SYSCFG {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const syscfg::RegisterBlock {
        1073807360 as *const _
    }
}
impl Deref for SYSCFG {
    type Target = syscfg::RegisterBlock;
    fn deref(&self) -> &syscfg::RegisterBlock {
        unsafe { &*SYSCFG::ptr() }
    }
}
#[doc = "System configuration controller"]
pub mod syscfg;
#[doc = "Operational amplifier"]
pub struct OPAMP {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for OPAMP {}
impl OPAMP {
    #[doc = r" Returns a pointer to the register block"]
    pub fn ptr() -> *const opamp::RegisterBlock {
        1073807416 as *const _
    }
}
impl Deref for OPAMP {
    type Target = opamp::RegisterBlock;
    fn deref(&self) -> &opamp::RegisterBlock {
        unsafe { &*OPAMP::ptr() }
    }
}
#[doc = "Operational amplifier"]
pub mod opamp;
#[allow(renamed_and_removed_lints)]
#[allow(private_no_mangle_statics)]
#[no_mangle]
static mut DEVICE_PERIPHERALS: bool = false;
#[doc = r" All the peripherals"]
#[allow(non_snake_case)]
pub struct Peripherals {
    #[doc = "GPIOA"]
    pub GPIOA: GPIOA,
    #[doc = "GPIOB"]
    pub GPIOB: GPIOB,
    #[doc = "GPIOC"]
    pub GPIOC: GPIOC,
    #[doc = "GPIOD"]
    pub GPIOD: GPIOD,
    #[doc = "GPIOE"]
    pub GPIOE: GPIOE,
    #[doc = "GPIOF"]
    pub GPIOF: GPIOF,
    #[doc = "TSC"]
    pub TSC: TSC,
    #[doc = "CRC"]
    pub CRC: CRC,
    #[doc = "FLASH"]
    pub FLASH: FLASH,
    #[doc = "RCC"]
    pub RCC: RCC,
    #[doc = "DMA1"]
    pub DMA1: DMA1,
    #[doc = "DMA2"]
    pub DMA2: DMA2,
    #[doc = "TIM2"]
    pub TIM2: TIM2,
    #[doc = "TIM3"]
    pub TIM3: TIM3,
    #[doc = "TIM4"]
    pub TIM4: TIM4,
    #[doc = "TIM15"]
    pub TIM15: TIM15,
    #[doc = "TIM16"]
    pub TIM16: TIM16,
    #[doc = "TIM17"]
    pub TIM17: TIM17,
    #[doc = "USART1"]
    pub USART1: USART1,
    #[doc = "USART2"]
    pub USART2: USART2,
    #[doc = "USART3"]
    pub USART3: USART3,
    #[doc = "UART4"]
    pub UART4: UART4,
    #[doc = "UART5"]
    pub UART5: UART5,
    #[doc = "SPI1"]
    pub SPI1: SPI1,
    #[doc = "SPI2"]
    pub SPI2: SPI2,
    #[doc = "SPI3"]
    pub SPI3: SPI3,
    #[doc = "I2S2EXT"]
    pub I2S2EXT: I2S2EXT,
    #[doc = "I2S3EXT"]
    pub I2S3EXT: I2S3EXT,
    #[doc = "EXTI"]
    pub EXTI: EXTI,
    #[doc = "COMP"]
    pub COMP: COMP,
    #[doc = "PWR"]
    pub PWR: PWR,
    #[doc = "CAN"]
    pub CAN: CAN,
    #[doc = "USB_FS"]
    pub USB_FS: USB_FS,
    #[doc = "I2C1"]
    pub I2C1: I2C1,
    #[doc = "I2C2"]
    pub I2C2: I2C2,
    #[doc = "IWDG"]
    pub IWDG: IWDG,
    #[doc = "WWDG"]
    pub WWDG: WWDG,
    #[doc = "RTC"]
    pub RTC: RTC,
    #[doc = "TIM6"]
    pub TIM6: TIM6,
    #[doc = "TIM7"]
    pub TIM7: TIM7,
    #[doc = "DAC"]
    pub DAC: DAC,
    #[doc = "DBGMCU"]
    pub DBGMCU: DBGMCU,
    #[doc = "TIM1"]
    pub TIM1: TIM1,
    #[doc = "TIM8"]
    pub TIM8: TIM8,
    #[doc = "ADC1"]
    pub ADC1: ADC1,
    #[doc = "ADC2"]
    pub ADC2: ADC2,
    #[doc = "ADC3"]
    pub ADC3: ADC3,
    #[doc = "ADC4"]
    pub ADC4: ADC4,
    #[doc = "ADC1_2"]
    pub ADC1_2: ADC1_2,
    #[doc = "ADC3_4"]
    pub ADC3_4: ADC3_4,
    #[doc = "SYSCFG"]
    pub SYSCFG: SYSCFG,
    #[doc = "OPAMP"]
    pub OPAMP: OPAMP,
}
impl Peripherals {
    #[doc = r" Returns all the peripherals *once*"]
    #[inline]
    pub fn take() -> Option<Self> {
        cortex_m::interrupt::free(|_| {
            if unsafe { DEVICE_PERIPHERALS } {
                None
            } else {
                Some(unsafe { Peripherals::steal() })
            }
        })
    }
    #[doc = r" Unchecked version of `Peripherals::take`"]
    pub unsafe fn steal() -> Self {
        debug_assert!(!DEVICE_PERIPHERALS);
        DEVICE_PERIPHERALS = true;
        Peripherals {
            GPIOA: GPIOA {
                _marker: PhantomData,
            },
            GPIOB: GPIOB {
                _marker: PhantomData,
            },
            GPIOC: GPIOC {
                _marker: PhantomData,
            },
            GPIOD: GPIOD {
                _marker: PhantomData,
            },
            GPIOE: GPIOE {
                _marker: PhantomData,
            },
            GPIOF: GPIOF {
                _marker: PhantomData,
            },
            TSC: TSC {
                _marker: PhantomData,
            },
            CRC: CRC {
                _marker: PhantomData,
            },
            FLASH: FLASH {
                _marker: PhantomData,
            },
            RCC: RCC {
                _marker: PhantomData,
            },
            DMA1: DMA1 {
                _marker: PhantomData,
            },
            DMA2: DMA2 {
                _marker: PhantomData,
            },
            TIM2: TIM2 {
                _marker: PhantomData,
            },
            TIM3: TIM3 {
                _marker: PhantomData,
            },
            TIM4: TIM4 {
                _marker: PhantomData,
            },
            TIM15: TIM15 {
                _marker: PhantomData,
            },
            TIM16: TIM16 {
                _marker: PhantomData,
            },
            TIM17: TIM17 {
                _marker: PhantomData,
            },
            USART1: USART1 {
                _marker: PhantomData,
            },
            USART2: USART2 {
                _marker: PhantomData,
            },
            USART3: USART3 {
                _marker: PhantomData,
            },
            UART4: UART4 {
                _marker: PhantomData,
            },
            UART5: UART5 {
                _marker: PhantomData,
            },
            SPI1: SPI1 {
                _marker: PhantomData,
            },
            SPI2: SPI2 {
                _marker: PhantomData,
            },
            SPI3: SPI3 {
                _marker: PhantomData,
            },
            I2S2EXT: I2S2EXT {
                _marker: PhantomData,
            },
            I2S3EXT: I2S3EXT {
                _marker: PhantomData,
            },
            EXTI: EXTI {
                _marker: PhantomData,
            },
            COMP: COMP {
                _marker: PhantomData,
            },
            PWR: PWR {
                _marker: PhantomData,
            },
            CAN: CAN {
                _marker: PhantomData,
            },
            USB_FS: USB_FS {
                _marker: PhantomData,
            },
            I2C1: I2C1 {
                _marker: PhantomData,
            },
            I2C2: I2C2 {
                _marker: PhantomData,
            },
            IWDG: IWDG {
                _marker: PhantomData,
            },
            WWDG: WWDG {
                _marker: PhantomData,
            },
            RTC: RTC {
                _marker: PhantomData,
            },
            TIM6: TIM6 {
                _marker: PhantomData,
            },
            TIM7: TIM7 {
                _marker: PhantomData,
            },
            DAC: DAC {
                _marker: PhantomData,
            },
            DBGMCU: DBGMCU {
                _marker: PhantomData,
            },
            TIM1: TIM1 {
                _marker: PhantomData,
            },
            TIM8: TIM8 {
                _marker: PhantomData,
            },
            ADC1: ADC1 {
                _marker: PhantomData,
            },
            ADC2: ADC2 {
                _marker: PhantomData,
            },
            ADC3: ADC3 {
                _marker: PhantomData,
            },
            ADC4: ADC4 {
                _marker: PhantomData,
            },
            ADC1_2: ADC1_2 {
                _marker: PhantomData,
            },
            ADC3_4: ADC3_4 {
                _marker: PhantomData,
            },
            SYSCFG: SYSCFG {
                _marker: PhantomData,
            },
            OPAMP: OPAMP {
                _marker: PhantomData,
            },
        }
    }
}
