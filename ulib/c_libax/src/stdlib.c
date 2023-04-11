#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include <libax.h>

void srand(unsigned s)
{
    ax_srand(s);
}

int rand(void)
{
    return ax_rand_u32();
}

#ifdef AX_CONFIG_ALLOC

void *malloc(size_t size)
{
    return ax_malloc(size);
}

void *realloc(void *memblock, size_t size)
{
    size_t o_size = *(size_t *)(memblock - 8);

    void *mem = ax_malloc(size);

    for (int i = 0; i < (o_size < size ? o_size : size); i++)
        ((char *)mem)[i] = ((char *)memblock)[i];

    ax_free(memblock);
    return mem;
}

void free(void *addr)
{
    return ax_free(addr);
}

#endif

_Noreturn void abort(void)
{
    ax_panic();
    __builtin_unreachable();
}

// TODO:
char *getenv(const char *name)
{
    unimplemented();
    return 0;
}

// TODO:
int __clzdi2(int a)
{
    return 0;
}
