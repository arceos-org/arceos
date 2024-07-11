# Dora on Arceos

* Dora (modified version to support Dora)
	* For ArceOS: https://github.com/arceos-org/dora/tree/noshmem-arceos
* Dora-benchmark
	* https://github.com/arceos-org/dora-benchmark/tree/rs-latency-dynamic
* ArceOS
	* https://github.com/arceos-org/arceos/tree/dora-wip

## 1. Compile and setup Dora on host

* In dora repo.

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

> Pay attention to `dora-node-api` dependency.
> We use "shmem" feature to control whether Dora uses shared memory. 
> When Dora is compiled with ArceOS unikernel, the "shmem" feature needs to be disabled.


```Toml
[dependencies]
dora-node-api = { git = "https://github.com/arceos-org/dora.git", branch = "noshmem-arceos", default-features = false }
```
