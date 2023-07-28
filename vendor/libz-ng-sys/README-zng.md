# libz-ng-sys

A library for linking zlib-ng (`libz-ng`) to Rust programs natively, rather
than in zlib-compat mode.

zlib-ng is a high-performance implementation of zlib. zlib-ng supports building
in two modes: zlib-compat mode, in whih it provides the same API as zlib and
generally works as a drop-in replacement, and native mode, in which it provides
its own API. The native API is almost identical to the zlib-compat API, except
that some types use more correct sizes (rather than the sizes required for zlib
compatibility), and the functions all have a `zng_` prefix. The latter allows
zlib and zlib-ng to coexist in the same program.

This crate provides bindings to the native zlib-ng API. However, for simplicity
of porting, this crate exports the same API as libz-sys (without the `zng_`
prefixes), making it easier to write Rust software compatible with both
libz-sys and libz-ng-sys.

# High-level API

This crate provides bindings to the raw low-level C API. For a higher-level
safe API to work with DEFLATE, zlib, or gzip streams, see
[`flate2`](https://docs.rs/flate2). `flate2` supports many different
implementations.

# Development

This crate is built from [the same sources as
`libz-sys`](https://github.com/rust-lang/libz-sys). From within those sources,
`Cargo.toml` is the manifest for `libz-sys`, and `Cargo-zng.toml` is the
manifest for `libz-ng-sys`. The script `./cargo-zng` invokes Cargo on a
temporary copy of the sources with `Cargo-zng.toml` replacing `Cargo.toml`; for
instance, use `./cargo-zng publish` to publish `libz-ng-sys`.

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
