#ifndef __SYS_MMAN_H__
#define __SYS_MMAN_H__

#include <sys/types.h>

#define PROT_READ  0x1 /* Page can be read.  */
#define PROT_WRITE 0x2 /* Page can be written.  */
#define PROT_EXEC  0x4 /* Page can be executed.  */
#define PROT_NONE  0x0 /* Page can not be accessed.  */
#define PROT_GROWSDOWN                      \
    0x01000000 /* Extend change to start of \
                  growsdown vma (mprotect only).  */
#define PROT_GROWSUP                        \
    0x02000000 /* Extend change to start of \
                  growsup vma (mprotect only).  */

/* Sharing types (must choose one and only one of these).  */
#define MAP_SHARED  0x01 /* Share changes.  */
#define MAP_PRIVATE 0x02 /* Changes are private.  */
#define MAP_SHARED_VALIDATE                         \
    0x03              /* Share changes and validate \
                         extension flags.  */
#define MAP_TYPE 0x0f /* Mask for type of mapping.  */

/* Other flags.  */
#define MAP_FIXED 0x10 /* Interpret addr exactly.  */
#define MAP_FILE  0
#ifdef __MAP_ANONYMOUS
#define MAP_ANONYMOUS __MAP_ANONYMOUS /* Don't use a file.  */
#else
#define MAP_ANONYMOUS 0x20 /* Don't use a file.  */
#endif
#define MAP_ANON MAP_ANONYMOUS
/* When MAP_HUGETLB is set bits [26:31] encode the log2 of the huge page size.  */
#define MAP_HUGE_SHIFT 26
#define MAP_HUGE_MASK  0x3f

#define MAP_FAILED ((void *)-1)

/* Flags for mremap.  */
#define MREMAP_MAYMOVE   1
#define MREMAP_FIXED     2
#define MREMAP_DONTUNMAP 4

void *mmap(void *addr, size_t len, int prot, int flags, int fildes, off_t off);
int munmap(void *addr, size_t length);
void *mremap(void *old_address, size_t old_size, size_t new_size, int flags,
             ... /* void *new_address */);
int mprotect(void *addr, size_t len, int prot);
int madvise(void *addr, size_t length, int advice);

#endif
