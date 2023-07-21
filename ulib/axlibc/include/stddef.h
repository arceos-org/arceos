#ifndef __STDDEF_H__
#define __STDDEF_H__

#include <stdint.h>
#include <sys/types.h>

/* size_t is used for memory object sizes */
typedef uintptr_t size_t;
typedef intptr_t ssize_t;
typedef ssize_t ptrdiff_t;

typedef long clock_t;
typedef int clockid_t;

#ifdef __cplusplus
#define NULL 0L
#else
#define NULL ((void *)0)
#endif

#if __GNUC__ > 3
#define offsetof(type, member) __builtin_offsetof(type, member)
#else
#define offsetof(type, member) ((size_t)((char *)&(((type *)0)->member) - (char *)0))
#endif

#endif // __STDDEF_H__
