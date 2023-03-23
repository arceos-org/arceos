#ifndef __STDLIB_H__
#define __STDLIB_H__

#include <stddef.h>

int rand(void);
void srand(unsigned);

#ifdef AX_CONFIG_ALLOC
void *malloc(size_t size);
void free(void *addr);
#endif

_Noreturn void abort(void);
char *getenv(const char *name);

#endif //__STDLIB_H__
