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
