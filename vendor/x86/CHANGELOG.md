# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

## [0.52.0] - 2022-10-18

- Add user-defined, hardware ignored bits to page-table flags.

## [0.51.0] - 2022-07-15

- Implement `core::iter::Step` for PAddr, VAddr, IOAddr types. This currently
  requires nightly so added a `unstable` Cargo feature to enable it
  conditionally.

## [0.50.0] - 2022-06-29

- `rdtscp` now returns a tuple in the form of `(cycles: u64, aux: u32)`, where
  `cycles` is the cycle count (as returned by this function in previous
  versions) and `aux` is the value of `IA32_TSC_AUX` -- which also gets read-out
  by `rdtscp`. If one prefers to use the old signature, the recommendation is to
  replace calls for `x86::time::rdtscp` with `core::arch::x86_64::__rdtscp`.
  Fixes #124.

## [0.49.0] - 2022-06-03

- Removed `x86::its64::segmentation::fs_deref()`: Users should replace calls to
  `fs_deref` with the more general `x86::bits64::segmentation::fs_deref!` macro.
  `fs_deref!(0)` is equivalent to `fs_deref()`.
- Removed `x86::bits64::segmentation::gs_deref()`: Users should replace calls to
  `gs_deref` with the more general `x86::bits64::segmentation::gs_deref!` macro.
  `gs_deref!(0)` is equivalent to `gs_deref()`.

## [0.48.0] - 2022-05-23

- Added `const new` constructor for X2APIC struct
- Use fully qualified `asm!` import for `int!` macro so clients do no longer
  need to import `asm!` themselves.
