# Test scripts

define unittest
  cargo test -p percpu $(1) -- --nocapture
  cargo test --workspace --exclude "arceos-*" $(1) -- --nocapture
endef
