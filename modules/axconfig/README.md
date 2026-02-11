# axconfig

Platform-specific constants and parameters for [ArceOS](https://github.com/arceos-org/arceos).

Uses [`axconfig-macros`](https://docs.rs/axconfig-macros) to generate compile-time configuration from a TOML file. Set the `AX_CONFIG_PATH` environment variable to point to a custom config; otherwise a built-in `dummy.toml` is used as fallback.

## Usage

```toml
[dependencies]
axconfig = "0.2"
```

## License

GPL-3.0-or-later OR Apache-2.0 OR MulanPSL-2.0
