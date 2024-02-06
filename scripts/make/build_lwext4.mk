### build libc.a for lwext4

# override FEATURES := fp_simd alloc paging fs fd
local_lib_feat := fp_simd alloc paging fs fd

export LIBC_BUILD_TARGET_DIR=$(abspath $(local_obj_dir))

### include scripts/make/build_c.mk
local_ulib_dir := ulib/axlibc
local_src_dir := $(local_ulib_dir)/c
local_inc_dir := $(local_ulib_dir)/include
local_obj_dir := $(local_ulib_dir)/lwext4_libc_$(ARCH)
lwext4_c_lib := $(local_obj_dir)/libc-$(ARCH).a

local_last_cflags := $(local_obj_dir)/.cflags

local_ulib_src := $(wildcard $(local_src_dir)/*.c)
local_ulib_hdr := $(wildcard $(local_inc_dir)/*.h)
local_ulib_obj := $(patsubst $(local_src_dir)/%.c,$(local_obj_dir)/%.o,$(local_ulib_src))

LOCAL_CFLAGS += $(addprefix -DAX_CONFIG_,$(shell echo $(local_lib_feat) | tr 'a-z' 'A-Z' | tr '-' '_'))
LOCAL_CFLAGS += -DAX_LOG_DEBUG

LOCAL_CFLAGS += -nostdinc -fno-builtin -ffreestanding -Wall
LOCAL_CFLAGS += -I$(CURDIR)/$(local_inc_dir)
# LOCAL_LDFLAGS += -nostdlib -static -no-pie --gc-sections -T$(LD_SCRIPT)

ifeq ($(MODE), release)
  LOCAL_CFLAGS += -O3
endif

ifeq ($(ARCH), x86_64)
  LOCAL_LDFLAGS += --no-relax
else ifeq ($(ARCH), riscv64)
  LOCAL_CFLAGS += -march=rv64gc -mabi=lp64d -mcmodel=medany
endif

# 在FEATURES中查找fp_simd, 判断为空。 SIMD单个指令同时处理多个寄存器数据，进行浮点运算
# ifeq ($(findstring fp_simd,$(FEATURES)),)
$(info FEATURES=$(FEATURES))

_check_lw_need_rebuild: $(local_obj_dir)
	@if [ "$(LOCAL_CFLAGS)" != "`cat $(local_last_cflags) 2>&1`" ]; then \
		echo "CFLAGS changed, rebuild"; \
		echo "$(LOCAL_CFLAGS)" > $(local_last_cflags); \
	fi

$(local_obj_dir):
	$(call run_cmd,mkdir,-p $@)

$(local_obj_dir)/%.o: $(local_src_dir)/%.c $(local_last_cflags)
	$(call run_cmd,$(CC),$(LOCAL_CFLAGS) -c -o $@ $<)

$(lwext4_c_lib): $(local_obj_dir) _check_lw_need_rebuild $(local_ulib_obj)
	$(call run_cmd,$(AR),rcs $@ $(local_ulib_obj))

lwext4_libc: gen_libc_header $(lwext4_c_lib) make_disk_image_ext4
	@echo local_lib_feat: $(local_lib_feat)
	@echo 'Built $(lwext4_c_lib) for lwext4'

ax_libc_header := $(local_inc_dir)/ax_pthread_mutex.h
gen_libc_header:
ifeq ($(wildcard $(ax_libc_header)),)
	@echo Generating libc header
	@echo "typedef struct { long __l[1]; } pthread_mutex_t;\n#define PTHREAD_MUTEX_INITIALIZER { .__l = {0}}" > $(ax_libc_header)
#@cargo build --target $(TARGET) --manifest-path $(local_ulib_dir)/Cargo.toml $(build_args-$(MODE)) --features fs
endif

disk_image_ext4 := $(CURDIR)/disk-ext4.img
make_disk_image_ext4:
ifeq ($(wildcard $(disk_image_ext4)),)
	@printf "Creating EXT4 disk image \"$(disk_image_ext4)\" ...\n"
	@dd if=/dev/zero of=$(disk_image_ext4) bs=1M count=128
	@mkfs.ext4 -t ext4 $(disk_image_ext4)
	@ln -s $(disk_image_ext4) $(CURDIR)/disk.img
else
	@printf "EXT4 disk image \"$(disk_image_ext4)\" exist\n"
endif

.PHONY: _check_lw_need_rebuild lwext4_libc
