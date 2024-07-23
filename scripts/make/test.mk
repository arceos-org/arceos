# Test scripts

define unit_test
  $(call run_cmd,cargo test,-p axfs $(1) --features "myfs" -- --nocapture)
  $(call run_cmd,cargo test,--workspace $(1) -- --nocapture)
endef
