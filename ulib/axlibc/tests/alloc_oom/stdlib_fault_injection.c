/* Exercise the real axlibc C implementation with a deterministic allocator. */
#include <assert.h>
#include <errno.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void *injected_malloc(size_t size);
static void injected_free(void *ptr);

#define AX_CONFIG_ALLOC
#define calloc arceos_calloc
#define realloc arceos_realloc
#define malloc injected_malloc
#define free injected_free
/* Only unrelated, unimplemented stdlib entry points use this logging macro. */
#define unimplemented(...) ((void)0)
#ifndef AXLIBC_STDLIB_SOURCE
#define AXLIBC_STDLIB_SOURCE "../../c/stdlib.c"
#endif
#include AXLIBC_STDLIB_SOURCE
#undef unimplemented
#undef free
#undef malloc
#undef realloc
#undef calloc

enum { BLOCK_COUNT = 8, BLOCK_BYTES = 256 };

struct block {
    size_t size;
    unsigned char bytes[BLOCK_BYTES];
};

_Static_assert(sizeof(size_t) == 8, "This test models axlibc's 8-byte header on a 64-bit host");
_Static_assert(offsetof(struct block, bytes) == 8, "The allocation header must precede data by 8 bytes");

static struct block blocks[BLOCK_COUNT];
static int used[BLOCK_COUNT];
static int fail_next;
static unsigned int malloc_calls;
static unsigned int free_calls;
static size_t last_size;

static void reset_allocator(void)
{
    memset(blocks, 0, sizeof(blocks));
    memset(used, 0, sizeof(used));
    fail_next = 0;
    malloc_calls = 0;
    free_calls = 0;
    last_size = 0;
    errno = 0;
}

static size_t block_index(const void *ptr)
{
    for (size_t i = 0; i < BLOCK_COUNT; ++i)
        if (ptr == blocks[i].bytes)
            return i;
    assert(!"invalid allocation pointer");
    return 0;
}

static void *injected_malloc(size_t size)
{
    ++malloc_calls;
    last_size = size;
    if (fail_next || size > BLOCK_BYTES) {
        fail_next = 0;
        errno = ENOMEM;
        return NULL;
    }
    for (size_t i = 0; i < BLOCK_COUNT; ++i) {
        if (!used[i]) {
            used[i] = 1;
            blocks[i].size = size;
            memset(blocks[i].bytes, 0xcc, BLOCK_BYTES);
            return blocks[i].bytes;
        }
    }
    assert(!"mock allocator pool exhausted");
    return NULL;
}

static void injected_free(void *ptr)
{
    if (!ptr)
        return;
    size_t i = block_index(ptr);
    assert(used[i]);
    used[i] = 0;
    ++free_calls;
    memset(blocks[i].bytes, 0xdd, BLOCK_BYTES);
}

static void fill_pattern(unsigned char *ptr, size_t size)
{
    for (size_t i = 0; i < size; ++i)
        ptr[i] = (unsigned char)(0x5a ^ i);
}

static void check_pattern(const unsigned char *ptr, size_t size)
{
    for (size_t i = 0; i < size; ++i)
        assert(ptr[i] == (unsigned char)(0x5a ^ i));
}

static void test_calloc_overflow(void)
{
    const size_t operands[][2] = {
        { SIZE_MAX / 2 + 1, 2 },
        { 2, SIZE_MAX / 2 + 1 },
        { SIZE_MAX, SIZE_MAX },
    };
    for (size_t i = 0; i < sizeof(operands) / sizeof(operands[0]); ++i) {
        reset_allocator();
        assert(arceos_calloc(operands[i][0], operands[i][1]) == NULL);
        assert(errno == ENOMEM);
        assert(malloc_calls == 0);
        assert(free_calls == 0);
    }

    reset_allocator();
    assert(arceos_calloc(SIZE_MAX, 1) == NULL);
    assert(malloc_calls == 1);
    assert(last_size == SIZE_MAX);
    assert(errno == ENOMEM);
}

static void test_calloc_failure(void)
{
    reset_allocator();
    fail_next = 1;
    assert(arceos_calloc(4, 8) == NULL);
    assert(malloc_calls == 1);
    assert(last_size == 32);
    assert(errno == ENOMEM);
    assert(free_calls == 0);
}

static void test_calloc_zeroing(void)
{
    reset_allocator();
    unsigned char *ptr = arceos_calloc(4, 7);
    assert(ptr != NULL);
    assert(last_size == 28);
    for (size_t i = 0; i < 28; ++i)
        assert(ptr[i] == 0);
    assert(ptr[28] == 0xcc);
    injected_free(ptr);
    assert(free_calls == 1);
}

static void test_calloc_zero_size(void)
{
    const size_t operands[][2] = { { 0, SIZE_MAX }, { SIZE_MAX, 0 }, { 0, 0 } };
    for (size_t i = 0; i < sizeof(operands) / sizeof(operands[0]); ++i) {
        reset_allocator();
        unsigned char *ptr = arceos_calloc(operands[i][0], operands[i][1]);
        assert(ptr != NULL);
        assert(last_size == 0);
        assert(ptr[0] == 0xcc);
        injected_free(ptr);

        reset_allocator();
        fail_next = 1;
        assert(arceos_calloc(operands[i][0], operands[i][1]) == NULL);
        assert(last_size == 0);
        assert(errno == ENOMEM);
    }
}

static void test_realloc_failure(void)
{
    reset_allocator();
    unsigned char *old = injected_malloc(16);
    fill_pattern(old, 16);
    size_t i = block_index(old);
    fail_next = 1;
    assert(arceos_realloc(old, 32) == NULL);
    assert(errno == ENOMEM);
    assert(malloc_calls == 2);
    assert(last_size == 32);
    assert(free_calls == 0);
    assert(used[i]);
    assert(blocks[i].size == 16);
    check_pattern(old, 16);
    injected_free(old);
    assert(free_calls == 1);
}

static void test_realloc_grow(void)
{
    reset_allocator();
    unsigned char *old = injected_malloc(16);
    fill_pattern(old, 16);
    size_t old_index = block_index(old);
    unsigned char *ptr = arceos_realloc(old, 32);
    assert(ptr != NULL);
    assert(ptr != old);
    check_pattern(ptr, 16);
    assert(ptr[16] == 0xcc);
    assert(last_size == 32);
    assert(!used[old_index]);
    assert(free_calls == 1);
    injected_free(ptr);
    assert(free_calls == 2);
}

static void test_realloc_shrink(void)
{
    reset_allocator();
    unsigned char *old = injected_malloc(16);
    fill_pattern(old, 16);
    unsigned char *ptr = arceos_realloc(old, 7);
    assert(ptr != NULL);
    check_pattern(ptr, 7);
    assert(ptr[7] == 0xcc);
    assert(last_size == 7);
    assert(free_calls == 1);
    injected_free(ptr);
}

static void test_realloc_null_input(void)
{
    reset_allocator();
    void *ptr = arceos_realloc(NULL, 24);
    assert(ptr != NULL);
    assert(malloc_calls == 1);
    assert(last_size == 24);
    assert(free_calls == 0);
    injected_free(ptr);

    reset_allocator();
    fail_next = 1;
    assert(arceos_realloc(NULL, 24) == NULL);
    assert(errno == ENOMEM);
    assert(malloc_calls == 1);
    assert(free_calls == 0);
}

static void test_realloc_zero_size(void)
{
    reset_allocator();
    unsigned char *old = injected_malloc(16);
    fill_pattern(old, 16);
    unsigned char *ptr = arceos_realloc(old, 0);
    assert(ptr != NULL);
    assert(last_size == 0);
    assert(ptr[0] == 0xcc);
    assert(free_calls == 1);
    injected_free(ptr);

    reset_allocator();
    old = injected_malloc(16);
    fill_pattern(old, 16);
    fail_next = 1;
    assert(arceos_realloc(old, 0) == NULL);
    assert(errno == ENOMEM);
    assert(free_calls == 0);
    assert(used[block_index(old)]);
    check_pattern(old, 16);
    injected_free(old);
}

int main(void)
{
    test_calloc_overflow();
    test_calloc_failure();
    test_calloc_zeroing();
    test_calloc_zero_size();
    test_realloc_failure();
    test_realloc_grow();
    test_realloc_shrink();
    test_realloc_null_input();
    test_realloc_zero_size();
    puts("PASS: axlibc calloc/realloc fault injection (9 scenario groups)");
    return 0;
}
