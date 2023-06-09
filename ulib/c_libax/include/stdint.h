#ifndef __STDINT_H__
#define __STDINT_H__

/* Explicitly-sized versions of integer types */
typedef char int8_t;
typedef unsigned char uint8_t;
typedef short int16_t;
typedef unsigned short uint16_t;
typedef int int32_t;
typedef unsigned int uint32_t;
typedef long long int64_t;
typedef unsigned long long uint64_t;

typedef int64_t int_fast64_t;

typedef int64_t intmax_t;
/* *
 * Pointers and addresses are 32 bits long.
 * We use pointer types to represent addresses,
 * uintptr_t to represent the numerical values of addresses.
 * */
#if __riscv_xlen == 64 || defined(__x86_64__) || defined(__aarch64__)
typedef int64_t intptr_t;
typedef uint64_t uintptr_t;
#elif __riscv_xlen == 32 || defined(__i386__)
typedef int32_t intptr_t;
typedef uint32_t uintptr_t;
#endif

#endif // __STDINT_H__
