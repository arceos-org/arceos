# Dora on Arceos

* Dora
	* For host: https://github.com/arceos-org/dora/tree/multi_machine
	* For ArceOS: https://github.com/arceos-org/dora/tree/arceos-porting
* Dora-benchmark
	* https://github.com/arceos-org/dora-benchmark/tree/rs-latency-dynamic
* ArceOS
	* https://github.com/arceos-org/arceos/tree/dora-wip

## 1. Compile and setup Dora on host

* In dora repo under `multi_machine` branch

```bash
# Build.
cargo build --release --all

# Dora up (coordinator & daemon).
./target/release/dora up
```

You should see output like:
```bash
started dora coordinator
started dora daemon
```

## 2. Start Dora based on `dataflow-dynamic.yml`.

* In dora-benchmark repo:  `/dora-benchmark/dora-rs/rs-latency-dynamic`
```bash
./PATH/TO/dora/you/just/compiled start dataflow-dynamic.yml
```

## 3. Boot Dora nodes upon ArceOS or in host.

For example, we start sink in host and node upon ArceOS.

* In dora-benchmark repo, start sink node:
```bash
./target/release/benchmark-example-sink-dynamic
```

* In ArceOS dir:
```bash
make A=apps/std/dora/node-dynamic SMP=1 NET=y BLK=y LOG=debug STD=y justrun
```

Pay attention to `dora-node-api` dependency,
this is just the `arceos-porting` branch of Dora repo,
it's a simplified and WIP version of Dora as a dependency of ArceOS std.
We have separated their dirs to facilitate the compilation of Dora in the host and Dora dependencies for ArceOS std.

```Toml
[workspace.dependencies]
# dora-node-api = "0.3.5" # { path = "../../../dora/apis/rust/node" }
dora-node-api = { path = "../../Dora/dora-arceos/apis/rust/node" }
```
