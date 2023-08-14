# numeric-enum-macro

A declarative macro for type-safe enum-to-numbers conversion. `no-std` supported!

```rust
use numeric_enum_macro::numeric_enum;

numeric_enum! {
    #[repr(i64)] // repr must go first.
    /// Some docs.
    ///
    /// Multiline docs works too.
    #[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)] // all the attributes are forwarded!
    pub enum Lol {
        // All the constants must have explicit values assigned!
        Kek = 14,
        Wow = 87,
    }
}
// Conversion to raw number:
assert_eq!(14i64, Lol::Kek.into());
// Conversion from raw number:
assert_eq!(Ok(Lol::Wow), Lol::try_from(87));
// Unknown number:
assert_eq!(Err(88), Lol::try_from(88));
```

License: MIT/Apache-2.0
