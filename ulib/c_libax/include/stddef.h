#ifndef __STDDEF_H__
#define __STDDEF_H__

#include <stdint.h>
#include <sys/types.h>

/* size_t is used for memory object sizes */
typedef uintptr_t size_t;
typedef intptr_t ssize_t;
typedef ssize_t ptrdiff_t;

typedef int clockid_t;

#ifdef __cplusplus
#define NULL 0L
#else
#define NULL ((void *)0)
#endif

#endif // __STDDEF_H__
