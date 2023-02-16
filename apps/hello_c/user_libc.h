#ifndef __USER_LIBC_H__
#define __USER_LIBC_H__

typedef unsigned long long size_t;
typedef size_t uintptr_t;
#define NULL 0

extern void dummy_syscall(size_t a0, size_t a1);

extern void* malloc(size_t size);
extern void free(void* addr);

#endif