#include <errno.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

/*
 * Run with a 128 MiB QEMU guest. The default 1 GiB request has a valid
 * Rust Layout, including axlibc's control block, but cannot fit in the guest.
 * Keep integer-limit requests separate: they exercise layout validation and
 * overflow, and can panic before the actual allocation failure is reached.
 *
 * APP_CFLAGS=-DMALLOC_OOM_CASE=1 selects SIZE_MAX.
 * APP_CFLAGS=-DMALLOC_OOM_CASE=2 selects PTRDIFF_MAX.
 */
#ifndef MALLOC_OOM_CASE
#define MALLOC_OOM_CASE 0
#endif

/* axlibc's stdint.h does not currently define PTRDIFF_MAX. */
#ifndef PTRDIFF_MAX
#define PTRDIFF_MAX ((ptrdiff_t)(((size_t)-1) >> 1))
#endif

#if MALLOC_OOM_CASE == 0
#define REQUEST_BYTES ((size_t)1 << 30)
#define CASE_NAME "valid-layout-1GiB"
#elif MALLOC_OOM_CASE == 1
#define REQUEST_BYTES ((size_t)SIZE_MAX)
#define CASE_NAME "SIZE_MAX"
#elif MALLOC_OOM_CASE == 2
#define REQUEST_BYTES ((size_t)PTRDIFF_MAX)
#define CASE_NAME "PTRDIFF_MAX"
#else
#error "MALLOC_OOM_CASE must be 0, 1, or 2"
#endif

#define SMALL_BYTES 64

static void fill_sentinel(unsigned char *ptr, unsigned char value)
{
    for (size_t i = 0; i < SMALL_BYTES; ++i) {
        ptr[i] = (unsigned char)(value ^ (unsigned char)i);
    }
}

static int check_sentinel(const unsigned char *ptr, unsigned char value)
{
    for (size_t i = 0; i < SMALL_BYTES; ++i) {
        if (ptr[i] != (unsigned char)(value ^ (unsigned char)i)) {
            return 0;
        }
    }
    return 1;
}

int main(void)
{
    unsigned char *before = malloc(SMALL_BYTES);
    unsigned char *warmup = malloc(SMALL_BYTES);
    if (before == NULL || warmup == NULL) {
        printf("malloc-oom: FAIL initial small allocation\n");
        free(before);
        free(warmup);
        return EXIT_FAILURE;
    }
    fill_sentinel(before, 0xa5);
    fill_sentinel(warmup, 0x3c);
    if (!check_sentinel(before, 0xa5) || !check_sentinel(warmup, 0x3c)) {
        printf("malloc-oom: FAIL initial sentinel\n");
        free(warmup);
        free(before);
        return EXIT_FAILURE;
    }
    free(warmup);
    free(NULL);

    /* Volatile keeps this an actual allocator call in optimized host checks. */
    volatile size_t request_bytes = REQUEST_BYTES;
    printf("malloc-oom: REQUEST %s (%zu bytes)\n", CASE_NAME, (size_t)request_bytes);
    errno = 0;
    void *large = malloc(request_bytes);
    if (large != NULL) {
        printf("malloc-oom: FAIL request succeeded; check guest memory size\n");
        free(large);
        free(before);
        return EXIT_FAILURE;
    }
    if (errno != ENOMEM) {
        printf("malloc-oom: FAIL errno after malloc\n");
        free(before);
        return EXIT_FAILURE;
    }
    printf("malloc-oom: returned NULL\n");

    if (!check_sentinel(before, 0xa5)) {
        printf("malloc-oom: FAIL live allocation changed after failure\n");
        free(before);
        return EXIT_FAILURE;
    }

    unsigned char *after = malloc(SMALL_BYTES);
    if (after == NULL) {
        printf("malloc-oom: FAIL small allocation after failure\n");
        free(before);
        return EXIT_FAILURE;
    }
    fill_sentinel(after, 0x5a);
    if (!check_sentinel(after, 0x5a) || !check_sentinel(before, 0xa5)) {
        printf("malloc-oom: FAIL sentinel after recovery\n");
        free(after);
        free(before);
        return EXIT_FAILURE;
    }
    free(after);
    free(before);

    errno = 0;
    void *zeroed = calloc(1, request_bytes);
    if (zeroed != NULL || errno != ENOMEM) {
        printf("malloc-oom: FAIL calloc OOM\n");
        free(zeroed);
        return EXIT_FAILURE;
    }
    volatile size_t max_size = (size_t)-1;
    errno = 0;
    zeroed = calloc(max_size, 2);
    if (zeroed != NULL || errno != ENOMEM) {
        printf("malloc-oom: FAIL calloc overflow\n");
        free(zeroed);
        return EXIT_FAILURE;
    }

    unsigned char *original = malloc(SMALL_BYTES);
    if (!original)
        return EXIT_FAILURE;
    fill_sentinel(original, 0x69);
    errno = 0;
    void *resized = realloc(original, request_bytes);
    if (resized != NULL) {
        printf("malloc-oom: FAIL realloc unexpectedly succeeded\n");
        free(resized);
        return EXIT_FAILURE;
    }
    if (errno != ENOMEM || !check_sentinel(original, 0x69)) {
        printf("malloc-oom: FAIL realloc changed old allocation\n");
        free(original);
        return EXIT_FAILURE;
    }
    free(original);

    unsigned char *small_zeroed = calloc(SMALL_BYTES, 1);
    if (!small_zeroed)
        return EXIT_FAILURE;
    for (size_t i = 0; i < SMALL_BYTES; ++i) {
        if (small_zeroed[i] != 0) {
            free(small_zeroed);
            return EXIT_FAILURE;
        }
    }
    fill_sentinel(small_zeroed, 0x3c);
    unsigned char *grown = realloc(small_zeroed, SMALL_BYTES * 2);
    if (!grown) {
        free(small_zeroed);
        return EXIT_FAILURE;
    }
    int preserved = check_sentinel(grown, 0x3c);
    free(grown);
    if (!preserved)
        return EXIT_FAILURE;

    /* Both NULL and a freeable pointer are acceptable for malloc(0). */
    void *zero = malloc(0);
    free(zero);
    free(NULL);
    printf("malloc-oom: PASS %s\n", CASE_NAME);
    return EXIT_SUCCESS;
}
