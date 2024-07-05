# Dora on Arceos


* on dora repo
```bash
./target/release/dora coordinator
DORA_NODE_CONFIG_PATH=`pwd` ./target/release/dora daemon

```

* on dora-benchmark
```bash
./PATH/TO/dora start dataflow.yml
```

* then you can get `rust-node.yml` and `rust-sink.yml` on dora repo.

* dora nodes on host
```bash
DORA_NODE_CONFIG=`cat rust-node.yml` ./target/release/benchmark-example-node
DORA_NODE_CONFIG=`cat rust-sink.yml` ./target/release/benchmark-example-sink
```

*dora nodes on ArceOS
```bash
# make sure sink.yml in your disk.img
make A=apps/std/dora/sink SMP=1 NET=y BLK=y LOG=debug STD=y build
# make sure node.yml in your disk.img
make A=apps/std/dora/node SMP=1 NET=y BLK=y LOG=debug STD=y build
```
