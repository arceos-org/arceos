# How to run ArceOS on raspi4

You have to download this tutorial first:

https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials

And follow this tutorial run the 06_uart_chainloader chapter.

Then move your ArceOS to its root directory.

Then choose `PLATFORM = raspi4-aarch64` in Makefile and use the command `make chainboot` to transmit the arceos.bin to your raspi4.
