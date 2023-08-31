#include <stddef.h>
#include <stdio.h>

// TODO:
void *mmap(void *addr, size_t len, int prot, int flags, int fildes, off_t off)
{
    unimplemented();
    return NULL;
}

// TODO:
int munmap(void *addr, size_t length)
{
    unimplemented();
    return 0;
}

// TODO:
void *mremap(void *old_address, size_t old_size, size_t new_size, int flags,
             ... /* void *new_address */)
{
    unimplemented();
    return NULL;
}

// TODO
int mprotect(void *addr, size_t len, int prot)
{
    unimplemented();
    return 0;
}

// TODO
int madvise(void *addr, size_t len, int advice)
{
    unimplemented();
    return 0;
}
