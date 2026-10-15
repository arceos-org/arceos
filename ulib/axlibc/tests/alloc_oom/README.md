# Allocation failure regression tests

From the ArceOS repository root:

```sh
python3 ulib/axlibc/tests/alloc_oom/run.py
python3 ulib/axlibc/tests/alloc_oom/run.py --release
sh ulib/axlibc/tests/alloc_oom/run_c_stdlib_tests.sh
```

The Rust probe compiles the actual axlibc `malloc.rs`, renaming its exported
symbols to avoid replacing the host C runtime's allocator. Oversized requests
are forwarded to the real axalloc allocator with a private 256 KiB heap.
Host diagnostics and small axlibc allocations use the system allocator.
The underlying allocator's small-allocation recovery is tested separately.
The larger page bitmap accommodates host address randomization without
increasing the heap. The probe provides a host errno shim.

Each case runs in a separate process. Allocation failures and invalid request
sizes must return NULL with ENOMEM. An infallible Rust Vec allocation must
still abort, so the expected abort is isolated and core dumps are disabled.
Both debug and release builds cover size arithmetic overflow.

The C test compiles the actual `stdlib.c` with allocation failure injection.
It checks calloc multiplication overflow, allocation failure and zeroing;
realloc failure preserving the original allocation; and successful copying
and freeing. AddressSanitizer and UndefinedBehaviorSanitizer check memory use.
The C fixture requires a 64-bit host, matching the existing 8-byte header ABI.

For an actual C application on a 128 MiB x86_64 guest:

```sh
make A=examples/malloc-oom-c ARCH=x86_64 MEM=128M ACCEL=n run
```

The application checks real OOM, ENOMEM, failure recovery, calloc and realloc.
It must print `malloc-oom: PASS valid-layout-1GiB` and shut down normally.
The default request is a valid 1 GiB allocation, which cannot fit in the guest.
`MALLOC_OOM_CASE=1` and `2` select SIZE_MAX and PTRDIFF_MAX. Force the application
object to rebuild when changing these flags:

```sh
make A=examples/malloc-oom-c ARCH=x86_64 MEM=128M ACCEL=n \
    APP_CFLAGS=-DMALLOC_OOM_CASE=1 -W examples/malloc-oom-c/main.c run
```

The allocation header and payload alignment are unchanged by this OOM fix.
