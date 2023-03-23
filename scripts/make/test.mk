# Test scripts

define unittest
  cargo test --workspace --exclude "arceos-*" --exclude "libax_bindings" $(1) -- --nocapture
endef
