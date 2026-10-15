#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
build_dir=$(mktemp -d "${TMPDIR:-/tmp}/arceos-c-stdlib.XXXXXX")
trap 'rm -f "$build_dir/stdlib_test"; rmdir "$build_dir"' EXIT HUP INT TERM

# The warnings suppressed below come from unrelated existing stdlib functions.
"${CC:-cc}" -std=gnu11 -O2 -Wall -Wextra -Werror \
    -Wno-unused-parameter -Wno-implicit-fallthrough -fno-builtin \
    -ffunction-sections -fdata-sections -Wl,--gc-sections \
    -fsanitize=address,undefined -fno-omit-frame-pointer \
    "$script_dir/stdlib_fault_injection.c" -o "$build_dir/stdlib_test"
"$build_dir/stdlib_test"
