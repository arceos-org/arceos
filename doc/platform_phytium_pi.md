# How to run ArceOS on phytium pi

First, we need `ostool` to build and upload the image to the board. It also supports windows.

```bash
cargo install ostool
```

If use windows, you need to install `docker-desktop`.

We also need to connect the board to the computer with serial port, and connect netwire to the board. The host pc and the board should be in the same network.

The pins of a USB to serial adapter need to be connected to the debugging serial port of the development board using jumper wires, noting that the receive and transmit lines should cross-connect:

a. Connect the GND (ground) pin of the USB to TTL module to the GND (ground) pin of the development board (pin 12).

b. Connect the RX (receive) pin of the USB to TTL module to the TX (transmit) pin of the development board (pin 8).

c. Connect the TX (transmit) pin of the USB to TTL module to the RX (receive) pin of the development board (pin 10).

![uart](./figures/phytium_uart.png)

Then, we can run it easily.

```bash
# cd arceos main dir.
ostool run uboot
```

![select](./figures/phytium_select_dtb.png)

We can ignore select dtb step by pressing `enter` directly. ArceOS dose not support dtb yet.

Then the cmdline will wait for you to put board power on or reset.

You can modify config in `.project.toml` to change the default behavior.

If everything goes well, you will see the following output:

![output](./figures/phytium_ok.png)
