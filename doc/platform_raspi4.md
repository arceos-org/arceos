# How to run ArceOS on raspi4

Recommand you download this tutorial first:

https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials

And follow this tutorial to run the 06_uart_chainloader chapter.
It will help you to connect your raspi4 to your computer and it will also teach you how to use `make chainboot` to run your code on raspi4.

Then run with features `ARCH=aarch64 PLATFORM = raspi4-aarch64` and use the command `make chainboot` to transmit the xxxx_raspi4-aarch64.bin to your raspi4.

# How to debug ArceOS on raspi4

Recommand you download this tutorial first:

https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials

And follow this tutorial to run the 08_hw_debug_JTAG chapter.
It will help you to connect your raspi4 to the JTAG and connect your JTAG to your computer and it will also teach you how to use `make jtagboot` to debug your raspi4.

Because the JTAG only support the first line of code be loaded at `0x80000` and only support single core, so you have to 
1. Change the file: /modules/axconfig/src/platform/raspi4_aarch64, replace the "kernel-base-vaddr" with "0x8_0000" and replace the "phys-virt-offset" with "0x0"
2. set the feature `SMP=1`

Then run with features `ARCH=aarch64 PLATFORM = raspi4-aarch64` and 
1. use the command `make jtagboot` to run a halt program on your raspi4 
2. start a new terminal, and run `make openocd` to connect your PC with the JTAG
3. start a new terminal, and run `make gdb` to start a gdb, and type `target remote :3333` to connect with your openocd, and type `load` to load the xxxx_raspi4-aarch64.bin to your raspi4 and start to debug.
