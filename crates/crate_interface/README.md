# crate_interface

[![Crates.io](https://img.shields.io/crates/v/crate_interface)](https://crates.io/crates/crate_interface)

Provides a way to **define** an interface (trait) in a crate, but can
**implement** or **use** it in any crate. It 's usually used to solve
the problem of *circular dependencies* between crates.

## Example

```rust
// Define the interface
#[crate_interface::def_interface]
pub trait HelloIf {
    fn hello(&self, name: &str, id: usize) -> String;
}

// Implement the interface in any crate
struct HelloIfImpl;

#[crate_interface::impl_interface]
impl HelloIf for HelloIfImpl {
    fn hello(&self, name: &str, id: usize) -> String {
        format!("Hello, {} {}!", name, id)
    }
}

// Call `HelloIfImpl::hello` in any crate
use crate_interface::call_interface;
assert_eq!(
    call_interface!(HelloIf::hello("world", 123)),
    "Hello, world 123!"
);
assert_eq!(
    call_interface!(HelloIf::hello, "rust", 456), // another calling style
    "Hello, rust 456!"
);
```
