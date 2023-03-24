# Test scripts

define unittest
  cargo test -p percpu $(1) -- --nocapture
  cargo test --workspace --exclude "arceos-*" --exclude "libax_bindings" $(1) -- --nocapture
endef
