# Build for linux

ARCH ?= x86_64

CC := $(ARCH)-linux-musl-gcc

CFLAGS :=
LDFLAGS := -static -no-pie

redis-version := 7.0.12
redis-dir := $(CURDIR)/redis-$(redis-version)
redis-objs := $(redis-dir)/src/redis-server.o
redis-bin := redis-server

redis-build-args := \
  CC=$(CC) \
  CFLAGS="$(CFLAGS)" \
  USE_JEMALLOC=no \
  -j

ifneq ($(V),)
  redis-build-args += V=$(V)
endif

all: build

$(redis-dir):
	@echo "Download redis source code"
	wget https://github.com/redis/redis/archive/$(redis-version).tar.gz -P $(APP)
	tar -zxvf $(APP)/$(redis-version).tar.gz -C $(APP) && rm -f $(APP)/$(redis-version).tar.gz
	cd $(redis-dir) && git init && git add .
	patch -p1 -N -d $(redis-dir) --no-backup-if-mismatch -r - < $(APP)/redis.patch

build: $(redis-dir)
	cd $(redis-dir) && $(MAKE) $(redis-build-args)
	$(CC) $(LDFLAGS) -o $(redis-bin) $(redis-objs)

run:
	./$(redis-bin)

clean:
	$(MAKE) -C $(redis-dir) distclean
	rm -f $(redis-bin)

.PHONY: all build clean
