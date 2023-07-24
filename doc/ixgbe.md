# How to run arceos with ixgbe NIC?

First, you need to enable a new network feature and comment out the existing feature in `modules/axruntime/Cargo.toml`:

```toml
# net = ["alloc", "paging", "axdriver/virtio-net", "dep:axnet"]
net = ["alloc", "paging", "axdriver/ixgbe", "dep:axnet"] # Ixgbe
```

Additionally, you also need to specify the platform that owns this network card. For example, we defined a toml file named x86_64-pc-oslab under the platforms directory to describe the platform characteristics.

You can use the following command to compile an 'httpserver' app application:

```shell
make A=apps/net/httpserver ARCH=x86_64 PLATFORM=x86_64-pc-oslab NET=y
```

You can also use the following command to start the iperf application:

```shell
make A=apps/c/iperf ARCH=x86_64 PLATFORM=x86_64-pc-oslab NET=y BLK=y
```