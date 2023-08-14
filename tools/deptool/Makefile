FEATURES ?= default
DEFAULT ?= y
FORMAT ?= mermaid
TARGET ?= helloworld
SAVE_PATH ?= output.txt
_DEFAULT_OPT =
_FEATURES_OPT =
BUILD_DIR = ./target

ifeq ($(DEFAULT), y)
	_DEFAULT_OPT = --no-default
endif

ifneq ($(FEATURES), default)
	_FEATURES_OPT = --name $(FEATURES)
endif
	
ifeq ($(TARGET),)
  $(error must specify a target using TARGET=... which should be a valid module, crate or app path)
endif

clean:
	cargo clean
	rm $(SAVE_PATH)

run:
	@cargo build
	@./target/debug/deptool $(_DEFAULT_OPT) \
	$(_FEATURES_OPT) \
	--format $(FORMAT) \
	--target $(TARGET) \
	--save-path $(SAVE_PATH)

.PHONY: build run clean
