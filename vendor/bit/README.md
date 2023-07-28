# Bit
[![crates.io version badge](https://img.shields.io/crates/v/bit.svg)](https://crates.io/crates/bit)

`bit` is a library which provides some useful helpers for dealing with bits and
bit ranges. For now it's just a rewrite of
[`rust-bit-field` crate](https://github.com/phil-opp/rust-bit-field), but more
features are planned. Some of them _could_ be:

- Support for arrays and slices.
- `bitflags`-like functionality.

# Usage
Add to your `Cargo.toml`:

```toml
[dependencies]
bit = "0.1"
```

And add to your code:

```rust
extern crate bit;
use bit::BitIndex;
```

# Example
```rust
extern crate bit;
use bit::BitIndex;

fn main() {
    let mut value = 0b11010110u8;

    // 8
    println!("{}", u8::bit_length());

    // true
    println!("{}", value.bit(1));

    // 0b10
    println!("{:#b}", value.bit_range(0..2));

    value
        .set_bit(3, true)
        .set_bit(2, false)
        .set_bit_range(5..8, 0b001);

    // 0b111010
    println!("{:#b}", value);
}
```
