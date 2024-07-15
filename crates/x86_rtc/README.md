# x86_rtc

[![Crates.io](https://img.shields.io/crates/v/x86_rtc)](https://crates.io/crates/x86_rtc)

System Real Time Clock (RTC) Drivers for x86_64 based on CMOS.

## Examples

```rust
use x86_rtc::Rtc;
let epoch_time = Rtc::new().get_unix_timestamp();
```
