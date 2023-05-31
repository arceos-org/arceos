# kernel_guard

[![Crates.io](https://img.shields.io/crates/v/kernel_guard)](https://crates.io/crates/kernel_guard)

RAII wrappers to create a critical section with local IRQs or preemption
disabled, used to implement spin locks in kernel.

The critical section is created after the guard struct is created, and is
ended when the guard falls out of scope.

The crate user must implement the `KernelGuardIf` trait using
[`crate_interface::impl_interface`](https://crates.io/crates/crate_interface) to provide the low-level implementantion
of how to enable/disable kernel preemption, if the feature `preempt` is
enabled.

Available guards:

- `NoOp`: Does nothing around the critical section.
- `IrqSave`: Disables/enables local IRQs around the critical section.
- `NoPreempt`: Disables/enables kernel preemption around the critical
section.
- `NoPreemptIrqSave`: Disables/enables both kernel preemption and local
IRQs around the critical section.

## Crate features

- `preempt`: Use in the preemptive system. If this feature is enabled, you
need to implement the `KernelGuardIf` trait in other crates. Otherwise
the preemption enable/disable operations will be no-ops. This feature is
disabled by default.

## Examples

```rust
use kernel_guard::{KernelGuardIf, NoPreempt};

struct KernelGuardIfImpl;

#[crate_interface::impl_interface]
impl KernelGuardIf for KernelGuardIfImpl {
    fn enable_preempt() {
        // Your implementation here
    }
    fn disable_preempt() {
        // Your implementation here
    }
}

let guard = NoPreempt::new();
/* The critical section starts here

Do something that requires preemption to be disabled

The critical section ends here */
drop(guard);
```
