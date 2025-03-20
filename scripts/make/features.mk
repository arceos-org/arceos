# Features resolving.
#
# Inputs:
#   - `FEATURES`: a list of features to be enabled split by spaces or commas.
#     The features can be selected from the crate `axfeat` or the user library
#     (crate `axstd` or `axlibc`).
#   - `APP_FEATURES`: a list of features to be enabled for the Rust app.
#
# Outputs:
#   - `AX_FEAT`: features to be enabled for ArceOS modules (crate `axfeat`).
#   - `LIB_FEAT`: features to be enabled for the user library (crate `axstd`, `axlibc`).
#   - `APP_FEAT`: features to be enabled for the Rust app.

ifeq ($(APP_TYPE),c)
  ax_feat_prefix := axfeat/
  lib_features := fp_simd irq alloc multitask fs net fd pipe select epoll
else
  ifeq ($(NO_AXSTD),y)
    ax_feat_prefix := axfeat/
  else
    ax_feat_prefix := axstd/
  endif
  lib_features :=
endif

lib_feat_prefix := $(AX_LIB)/

override FEATURES := $(shell echo $(FEATURES) | tr ',' ' ')

ifeq ($(APP_TYPE), c)
  ifneq ($(wildcard $(APP)/features.txt),)    # check features.txt exists
    override FEATURES += $(shell cat $(APP)/features.txt)
  endif
  ifneq ($(filter fs net pipe select epoll,$(FEATURES)),)
    override FEATURES += fd
  endif
endif

override FEATURES := $(strip $(FEATURES))

ax_feat :=
lib_feat :=

ifneq ($(filter $(LOG),off error warn info debug trace),)
  ax_feat += log-level-$(LOG)
else
  $(error "LOG" must be one of "off", "error", "warn", "info", "debug", "trace")
endif

ifeq ($(BUS),mmio)
  ax_feat += bus-mmio
endif

ifeq ($(shell test $(SMP) -gt 1; echo $$?),0)
  lib_feat += smp
endif

ax_feat += $(filter-out $(lib_features),$(FEATURES))
lib_feat += $(filter $(lib_features),$(FEATURES))

AX_FEAT := $(strip $(addprefix $(ax_feat_prefix),$(ax_feat)))
LIB_FEAT := $(strip $(addprefix $(lib_feat_prefix),$(lib_feat)))
APP_FEAT := $(strip $(shell echo $(APP_FEATURES) | tr ',' ' '))
