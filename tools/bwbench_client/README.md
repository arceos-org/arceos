# Benchmark BandWidth Client

Benchmark BandWidth Client is a performance testing tool for measuring the network card's ability to send Ethernet packets. It can test both the transmission throughput and the reception throughput.

## Usage
In client:
```shell
cargo run --release [sender|receiver] [interface]
```

By reading the source code, you can control the behavior of the benchmark by modifying constants such as `MAX_BYTES`.

In arceos: 
```
make A=apps/net/bwbench NET=y run
```

By default, arceos `bebench` uses `bench_transmit`. You can uncomment the line and add `bench_receive`, but please note that currently only one of either `bench_transmit` or `bench_receive` is allowed to be enabled.
