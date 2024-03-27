#[allow(unused_macros)]
macro_rules! impl_uintlike_and_shift {
    ($reg:ident) => {
        impl UIntLike for $reg {
            fn zero() -> Self {
                $reg::from_bits_truncate(0)
            }
        }

        impl Shl<usize> for $reg {
            type Output = Self;
        
            fn shl(self, rhs: usize) -> Self::Output {
                $reg::from_bits_truncate(self.bits() << rhs)
            }
        }

        impl Shr<usize> for $reg {
            type Output = Self;
        
            fn shr(self, rhs: usize) -> Self::Output {
                $reg::from_bits_truncate(self.bits() >> rhs)
            }
        }
    };
}

pub(crate) mod gicv2_regs;
pub(crate) mod gicv3_regs;
