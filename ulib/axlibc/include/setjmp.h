#ifndef _SETJMP_H
#define _SETJMP_H

#include <features.h>

#if defined(__aarch64__)
typedef unsigned long __jmp_buf[22];
#elif defined(__riscv__) || defined(__riscv)
typedef unsigned long __jmp_buf[26];
#elif defined(__x86_64__)
typedef unsigned long __jmp_buf[8];
#elif defined(__loongarch__)
typedef unsigned long __jmp_buf[21];
#endif

typedef struct __jmp_buf_tag {
    __jmp_buf __jb;
    unsigned long __fl;
    unsigned long __ss[128 / sizeof(long)];
} jmp_buf[1];

int setjmp(jmp_buf);
_Noreturn void longjmp(jmp_buf, int);

#endif
