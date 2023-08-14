//! A declarative macro for type-safe enum-to-numbers conversion. `no-std` supported!
//!
//! ```
//! use numeric_enum_macro::numeric_enum;
//!
//! numeric_enum! {
//!     #[repr(i64)] // repr must go first.
//!     /// Some docs.
//!     ///
//!     /// Multiline docs works too.
//!     #[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)] // all the attributes are forwarded!
//!     pub enum Lol {
//!         // All the constants must have explicit values assigned!
//!         Kek = 14,
//!         Wow = 87,
//!     }
//! }
//!
//! const KEK: u32 = 0;
//! const WOW: u32 = 1;
//!
//! numeric_enum! {
//!     #[repr(u32)] // repr must go first.
//!     /// Some docs.
//!     ///
//!     /// Multiline docs works too.
//!     #[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)] // all the attributes are forwarded!
//!     pub enum Lol2 {
//!         /// This is KEK
//!         Kek = KEK,
//!         /// And this is WOW
//!         Wow = WOW,
//!     }
//! }
//!
//! # use ::core::convert::TryFrom;
//! // Conversion to raw number:
//! assert_eq!(14i64, Lol::Kek.into());
//! // Conversion from raw number:
//! assert_eq!(Ok(Lol::Wow), Lol::try_from(87));
//! // Unknown number:
//! assert_eq!(Err(88), Lol::try_from(88));
//!
//! assert_eq!(Ok(Lol2::Wow), Lol2::try_from(WOW));
//! ```

#![no_std]

/// Declares an enum with a given numeric representation defined by literals.
///
/// Only explicitly enumerated enum constants are supported.
///
/// Automatically derives `TryFrom<$repr>` and `From<$name>`.
///
/// For examples look at the crate root documentation.
#[macro_export]
macro_rules! numeric_enum {
    (#[repr($repr:ident)]
     $(#$attrs:tt)* $vis:vis enum $name:ident {
        $($(#$enum_attrs:tt)* $enum:ident = $constant:expr),* $(,)?
    } ) => {
        #[repr($repr)]
        $(#$attrs)*
        $vis enum $name {
            $($(#$enum_attrs)* $enum = $constant),*
        }

        impl ::core::convert::TryFrom<$repr> for $name {
            type Error = $repr;

            fn try_from(value: $repr) -> ::core::result::Result<Self, $repr> {
                $(if $constant == value { return Ok($name :: $enum); } )*
                Err(value)
            }
        }

        impl ::core::convert::From<$name> for $repr {
            fn from(value: $name) -> $repr {
                match value {
                    $($name :: $enum => $constant,)*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    numeric_enum! {
        #[repr(i16)]
        /// Documentation.
        ///
        /// Multiline.
        #[derive(Debug, PartialEq, Eq)]
        pub enum PublicEnum { Zero = 0, Lol = -1 }
    }

    numeric_enum! {
        #[repr(u8)]
        enum TrailingComa { A = 0, B = 1, }
    }

    numeric_enum! {
        #[repr(u8)]
        enum NoTrailingComa { A = 0, B = 1 }
    }

    const ZERO: u8 = 0;
    const LOL: u8 = 1;

    numeric_enum! {
        #[repr(u8)]
        enum PrivateEnum {
            Zero = ZERO,
            Lol = LOL,
        }
    }

    #[test]
    fn it_works() {
        use core::convert::TryFrom;

        assert_eq!(-1i16, PublicEnum::Lol.into());
        assert_eq!(PublicEnum::try_from(0), Ok(PublicEnum::Zero));
        assert_eq!(PublicEnum::try_from(-1), Ok(PublicEnum::Lol));
        assert_eq!(PublicEnum::try_from(2), Err(2));
    }
}
