# riscv_goldfish

[![Crates.io](https://img.shields.io/crates/v/riscv_goldfish)](https://crates.io/crates/riscv_goldfish)

System Real Time Clock (RTC) Drivers for riscv based on goldfish.

## Examples

```rust
use riscv_goldfish::Rtc;

let epoch_time = Rtc::new(0x10_1000).get_unix_timestamp();
```

`base_addr` needs to be the device virtual address available for mmio, which can be obtained from the device tree, for example:

```
soc {
    #address-cells = <0x02>;
    #size-cells = <0x02>;
    compatible = "simple-bus";
    ranges;

    rtc@101000 {
        interrupts = <0x0b>;
        interrupt-parent = <0x03>;
        reg = <0x00 0x101000 0x00 0x1000>;
        compatible = "google,goldfish-rtc";
    };

    ...
}
```