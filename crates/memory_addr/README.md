# memory_addr

[![Crates.io](https://img.shields.io/crates/v/memory_addr)](https://crates.io/crates/memory_addr)

Wrappers and helper functions for physical and virtual memory addresses.

## Examples

```rust
use memory_addr::{PhysAddr, VirtAddr};

let phys_addr = PhysAddr::from(0x12345678);
let virt_addr = VirtAddr::from(0x87654321);

assert_eq!(phys_addr.align_down(0x1000usize), PhysAddr::from(0x12345000));
assert_eq!(phys_addr.align_offset_4k(), 0x678);
assert_eq!(virt_addr.align_up_4k(), VirtAddr::from(0x87655000));
assert!(!virt_addr.is_aligned_4k());
assert!(VirtAddr::from(0xabcedf0).is_aligned(16usize));
```
