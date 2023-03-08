#include <stdint.h>
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

void *malloc(size_t size) {
    return ax_malloc(size);
}

void free(void *addr) {
    return ax_free(addr);
}

#endif

_Noreturn void abort(void)
{
    ax_panic();
    __builtin_unreachable();
}
