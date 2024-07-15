# arm_pl031

[![Crates.io](https://img.shields.io/crates/v/arm_pl031)](https://crates.io/crates/arm_pl031)

System Real Time Clock (RTC) Drivers for aarch64 based on PL031.

## Examples

```rust
use arm_pl031::Rtc;

let epoch_time = Rtc::new(0x901_0000).get_unix_timestamp();
```

`base_addr` needs to be the device virtual address available for mmio, which can be obtained from the device tree, for example:

```
/ {
	interrupt-parent = <0x8002>;
	model = "linux,dummy-virt";
	#size-cells = <0x02>;
	#address-cells = <0x02>;
	compatible = "linux,dummy-virt";

	pl031@9010000 {
		clock-names = "apb_pclk";
		clocks = <0x8000>;
		interrupts = <0x00 0x02 0x04>;
		reg = <0x00 0x9010000 0x00 0x1000>;
		compatible = "arm,pl031\0arm,primecell";
	};

    ...
}
```