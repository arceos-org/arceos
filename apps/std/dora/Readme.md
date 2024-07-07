# Dora on Arceos

* Dora
	* For host: https://github.com/arceos-org/dora/tree/modify_spawn
	* For ArceOS: https://github.com/arceos-org/dora/tree/arceos-porting
* ArceOS
	* https://github.com/arceos-org/arceos/tree/dora-wip

## 1. Setup Dora coordinator and daemon on host

* In dora repo under `modify_spawn` branch

```bash
# Build.
cargo build --release --all

# Run coordinator.
./target/release/dora coordinator

# Run daemon.
DORA_NODE_CONFIG_PATH=`pwd` ./target/release/dora daemon
```

## 2. Generate configuration file for Dora nodes.

* In dora-benchmark repo:  `/dora-benchmark/dora-rs/rs-latency`
```bash
./PATH/TO/dora/you/just/compiled start dataflow.yml
```

* then you can get `rust-node.yml` and `rust-sink.yml` on dora repo.

## 3. Boot Dora nodes based on these configuration file.

### To boot Dora nodes on host
```bash
# Build.
cargo build --release --all

# Run nodes.
DORA_NODE_CONFIG=`cat rust-node.yml` ./target/release/benchmark-example-node
DORA_NODE_CONFIG=`cat rust-sink.yml` ./target/release/benchmark-example-sink
```

### To boot Dora nodes upon ArceOS

In ArceOS dir under `dora-wip` branch.

Pay attention to `dora-node-api` dependency,
this is just the `arceos-porting` branch of Dora repo,
We have separated their dirs to facilitate the compilation of Dora in the host and Dora dependencies for ArceOS std.

```Toml
[workspace.dependencies]
# dora-node-api = "0.3.5" # { path = "../../../dora/apis/rust/node" }
dora-node-api = { path = "../../Dora/dora-arceos/apis/rust/node" }
```

* Run ArceOS on QEMU/KVM
	* make `disk.img` for ArceOS, make sure it contains the configuration files just generated for the Dora nodes.
	* change `socket_addr` to : `socket_addr: "10.0.2.2:44631"` in configuration files.

```bash
# make sure sink.yml in your disk.img
make A=apps/std/dora/sink SMP=1 NET=y BLK=y LOG=debug STD=y build
# make sure node.yml in your disk.img
make A=apps/std/dora/node SMP=1 NET=y BLK=y LOG=debug STD=y build
```
