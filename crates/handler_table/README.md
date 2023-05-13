# handler_table

[![Crates.io](https://img.shields.io/crates/v/handler_table)](https://crates.io/crates/handler_table)

A lock-free table of event handlers.

## Examples

```rust
use handler_table::HandlerTable;

static TABLE: HandlerTable<8> = HandlerTable::new();

TABLE.register_handler(0, || {
   println!("Hello, event 0!");
});
TABLE.register_handler(1, || {
   println!("Hello, event 1!");
});

assert!(TABLE.handle(0)); // print "Hello, event 0!"
assert!(!TABLE.handle(2)); // unregistered
```
