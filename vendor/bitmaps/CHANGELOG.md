# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) and this project
adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [3.2.0] - 2022-04-30

### ADDED

-   `Bitmap` now implements `TryFrom<&[u8]>` and `AsRef<[u8]>`, and an `as_bytes()` method was
    added, to facilitate conversion to and from byte arrays.
-   The iterator is now bidirectional, and `next_index`, `prev_index` and `last_index` methods, with
    corresponding `false_index` variants, have been added. (#14)

## [3.1.0] - 2021-04-30

### ADDED

-   The methods `first_false_index()` and `is_full()` have been added to `Bitmap`. (#12)

### FIXED

-   The previous version broke the `no_std` feature; it has now been restored. (#11)

## [3.0.0] - 2021-04-26

### CHANGED

-   This crate now uses const generics, rather than the `typenum` crate, to encode numerical values
    in types. This means you can now use plain integers in the `Bitmap` type, eg. `Bitmap<32>`,
    rather than the old `Bitmap<typenum::U32>`. (#8, #9)

### ADDED

-   `Bitmap` now implements `Hash`, `Eq`, `PartialOrd` and `Ord`. (#7)
-   The `as_value()` method has been added to `Bitmap`, to get a reference to the underlying value.
    (#7)
-   `bitmaps::Iter` now implements `Clone` and `Debug`. (#4)

## [2.1.0] - 2020-03-26

### ADDED

-   There is now a `std` feature flag, on by default, which you can disable to get a `no_std` crate.

## [2.0.0] - 2019-09-09

### CHANGED

-   `Bits` now does a lot less work, which is now being done instead by the `BitOps` trait on its
    storage type. This turns out to improve compilation time quite considerably. If you were using
    methods on `Bits` directly, they will have moved to `BitOps`.
-   `Debug` now prints a single hex value for the entire bitmap, rather than deferring to the
    storage type.
-   `Iter` now takes a reference instead of a copy, which is more sensible for larger bitmaps.

### ADDED

-   `Bitmap` now implements `BitAnd`, `BitOr`, `BitXor`, their equivalent assignation traits, and
    `Not`, meaning you can now use bitwise operators on them, even the very big array-of-u128 ones.
-   A `Bitmap::mask()` constructor has been added, to construct bitmasks more efficiently, now that
    there are bitwise operators to use them with.

## [1.0.0] - 2019-09-06

Initial release.
