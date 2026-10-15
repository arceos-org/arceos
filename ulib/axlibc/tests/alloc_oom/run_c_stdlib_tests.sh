#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
build_dir=$(mktemp -d "${TMPDIR:-/tmp}/arceos-c-stdlib.XXXXXX")
trap 'rm -f "$build_dir/stdlib_test" "$build_dir/fopen_test" "$build_dir/fopen_test.o"; rmdir "$build_dir"' EXIT HUP INT TERM

# The warnings suppressed below come from unrelated existing stdlib functions.
"${CC:-cc}" -std=gnu11 -O2 -Wall -Wextra -Werror \
    -Wno-unused-parameter -Wno-implicit-fallthrough -fno-builtin \
    -ffunction-sections -fdata-sections -Wl,--gc-sections \
    -fsanitize=address,undefined -fno-omit-frame-pointer \
    "$script_dir/stdlib_fault_injection.c" -o "$build_dir/stdlib_test"
"$build_dir/stdlib_test"

# Use axlibc's headers for its FILE layout. Hidden symbols and section GC keep
# unrelated stdio definitions from replacing the host runtime's functions.
"${CC:-cc}" -std=gnu11 -O2 -Wall -Wno-unused-parameter \
    -nostdinc -I"$script_dir/../../include" -fno-builtin -ffreestanding \
    -fvisibility=hidden -ffunction-sections -fdata-sections \
    -fsanitize=address,undefined -fno-omit-frame-pointer \
    -c "$script_dir/fopen_failure.c" -o "$build_dir/fopen_test.o"
objcopy --localize-hidden "$build_dir/fopen_test.o"
"${CC:-cc}" -fsanitize=address,undefined -Wl,--gc-sections \
    "$build_dir/fopen_test.o" -o "$build_dir/fopen_test"
"$build_dir/fopen_test"
echo "PASS: fopen allocation failure preserves ENOMEM without opening a file"
