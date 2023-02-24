#ifndef __STDDEF_H__
#define __STDDEF_H__

#include <stdint.h>

/* size_t is used for memory object sizes */
typedef uintptr_t size_t;
typedef intptr_t ssize_t;

typedef int pid_t;

#define NULL ((void *)0)

#endif // __STDDEF_H__
