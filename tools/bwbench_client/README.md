# Benchmark BandWidth Client

Benchmark BandWidth Client is a performance testing tool for measuring the network card's ability to send Ethernet packets. It can test both the transmission throughput and the reception throughput.

## Usage
In client:
```shell
cargo build --release
sudo ./target/release/bwbench_client [sender|receiver] [interface]
```

By reading the source code, you can control the behavior of the benchmark by modifying constants such as `MAX_BYTES`.

In arceos:

```shell
make A=apps/net/bwbench LOG=info NET=y run
```

By default, arceos `bebench` uses `bench_transmit`. You can uncomment the line and add `bench_receive`, but please note that currently only one of either `bench_transmit` or `bench_receive` is allowed to be enabled.


## Example: benchmark bandwidth of QEMU tap netdev

In client:

```shell
cargo build --release
sudo ./scripts/net/qemu-tap-ifup.sh enp8s0
sudo ./target/release/bwbench_client [sender|receiver] tap0
```

In arceos:

```shell
make A=apps/net/bwbench LOG=info NET=y NET_DEV=tap run
```
