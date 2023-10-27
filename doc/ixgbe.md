# How to run arceos with ixgbe NIC?

You need to specify the platform that owns this network card. For example, we defined a toml file named `x86_64-pc-oslab`` under the platforms directory to describe the platform characteristics.

You can use the following command to compile an 'httpserver' app application:

```shell
make A=apps/net/httpserver PLATFORM=x86_64-pc-oslab FEATURES=driver-ixgbe
```

You can also use the following command to start the iperf application:

```shell
make A=apps/c/iperf PLATFORM=x86_64-pc-oslab FEATURES=driver-ixgbe,driver-ramdisk
```

## Use ixgbe NIC in QEMU with PCI passthrough

1. Install the `vfio-pci` driver in the host:

    ```shell
    sudo modprobe vfio-pci
    ```

2. Bind the NIC to the `vfio-pci` driver (assume the PCI address is `02:00.0`):

    ```shell
    sudo ./scripts/net/pci-bind.sh vfio-pci 02:00.0
    # Equivalent to:
    # echo 0000:02:00.0 > /sys/bus/pci/drivers/ixgbe/unbind
    # echo vfio-pci > /sys/bus/pci/devices/0000:02:00.0/driver_override
    # echo 0000:02:00.0 > /sys/bus/pci/drivers/vfio-pci/bind
    ```

3. Build and run ArceOS:

    ```shell
    make A=apps/net/httpserver FEATURES=driver-ixgbe VFIO_PCI=02:00.0 IP=x.x.x.x GW=x.x.x.x run
    ```

4. If no longer in use, bind the NIC back to the `ixgbe` driver:

    ```shell
    sudo ./scripts/net/pci-bind.sh ixgbe 02:00.0
    # Equivalent to:
    # echo 0000:02:00.0 > /sys/bus/pci/drivers/vfio-pci/unbind
    # echo ixgbe > /sys/bus/pci/devices/0000:02:00.0/driver_override
    # echo 0000:02:00.0 > /sys/bus/pci/drivers/ixgbe/bind
    ```
