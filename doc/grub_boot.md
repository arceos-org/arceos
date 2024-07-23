# Boot ArceOS on the x86 PC with GRUB

## 1. Build ArceOS and generate the kernel image

Assume the application is located at `path/to/awesomeapp`, and the target platform is `x86_64-pc-oslab` (the configuration file is located at [platforms/x86_64-pc-oslab.toml](../platforms/x86_64-pc-oslab.toml)):

```shell
make A=path/to/awesomeapp ARCH=x86_64 PLATFORM=x86_64-pc-oslab
```

The ELF format kernel image will be generated at `path/to/awesomeapp/awesomeapp-x86_64-pc-oslab.elf`.

## 2. Copy the kernel image to the `/boot` directory on the target machine

```shell
sudo cp path/to/awesomeapp/awesomeapp-x86_64-pc-oslab.elf /boot
```

## 3. Add boot entry to GRUB

Append the following lines to the `/etc/grub.d/40_custom` file on the target machine:

```shell
submenu 'Boot ArceOS with multiboot' {
  menuentry 'Boot ArceOS awesomeapp' {
    echo 'ArceOS awesomeapp is booting...'
    multiboot /boot/awesomeapp-x86_64-pc-oslab.elf
  }
}
```

## 4. Update GRUB configuration and reboot the machine

```shell
sudo update-grub2
sudo reboot
```

After rebooting, select the corresponding entry in the GRUB menu to boot ArceOS.

## 5. View the console output

ArceOS will print the log via the serial port.
You need to use a [USB-to-serial adapter](https://en.wikipedia.org/wiki/USB-to-serial_adapter) to connect the serial port of the target machine to another machine.
After that, a character device file will be created on that machine, such as `/dev/ttyUSB0`.
Then you can use the `screen` command to view the output and send commands to ArceOS:

```shell
screen /dev/ttyUSB0 115200
```
