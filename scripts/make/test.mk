# Test scripts

define unit_test
  cargo test -p percpu $(1) -- --nocapture
  cargo test -p axfs $(1) --features "myfs" -- --nocapture
endef

define app_test
  $(CURDIR)/scripts/test/app_test.sh
endef
