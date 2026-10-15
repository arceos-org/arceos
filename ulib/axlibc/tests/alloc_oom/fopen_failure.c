/* Compile the real stdio implementation with its own FILE definition. */
#define AX_CONFIG_FS
#define malloc injected_malloc
#define open injected_open
#define fopen arceos_fopen
#ifndef AXLIBC_STDIO_SOURCE
#define AXLIBC_STDIO_SOURCE "../../c/stdio.c"
#endif
#include AXLIBC_STDIO_SOURCE
#undef fopen
#undef open
#undef malloc

static unsigned int malloc_calls;
static unsigned int open_calls;

void *injected_malloc(size_t size)
{
    ++malloc_calls;
    if (size != sizeof(FILE))
        __builtin_trap();
    errno = ENOMEM;
    return NULL;
}

int injected_open(const char *filename, int flags, ...)
{
    (void)filename;
    (void)flags;
    ++open_calls;
    errno = EACCES;
    return -1;
}

__attribute__((visibility("default"))) int main(void)
{
    errno = 0;
    FILE *file = arceos_fopen("unused", "w");
    if (file != NULL || errno != ENOMEM || malloc_calls != 1 || open_calls != 0)
        return 1;

    /* Invalid modes must still fail before allocating or opening a file. */
    errno = 0;
    file = arceos_fopen("unused", "invalid");
    if (file != NULL || errno != EINVAL || malloc_calls != 1 || open_calls != 0)
        return 1;
    return 0;
}
