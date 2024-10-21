# How to run ArceOS on Phytium Pi

See more details in [this doc](https://chenlongos.com/Phytium-Car/ch1-1.html).

Build ArceOS: `make A=examples/helloworld PLATFORM=aarch64-phytium-pi LOG=trace`.

Prepare a USB flash disk and copy `examples/helloworld/helloworld_aarch64-phytium-pi.bin` to it.

Stop autoboot in U-Boot and execute following commands:

```
Phytium-Pi# usb start
Phytium-Pi# fatload usb 0:2 0x90100000 helloworld_aarch64-phytium-pi.bin
Phytium-Pi# go 0x90100000
```
