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

#define INT8_MIN  (-1 - 0x7f)
#define INT16_MIN (-1 - 0x7fff)
#define INT32_MIN (-1 - 0x7fffffff)
#define INT64_MIN (-1 - 0x7fffffffffffffff)

#define INT8_MAX  (0x7f)
#define INT16_MAX (0x7fff)
#define INT32_MAX (0x7fffffff)
#define INT64_MAX (0x7fffffffffffffff)

#define UINT8_MAX  (0xff)
#define UINT16_MAX (0xffff)
#define UINT32_MAX (0xffffffffu)
#define UINT64_MAX (0xffffffffffffffffu)

#define INTPTR_MIN  INT64_MIN
#define INTPTR_MAX  INT64_MAX
#define UINTPTR_MAX UINT64_MAX

/* *
 * Pointers and addresses are 32 bits long.
 * We use pointer types to represent addresses,
 * uintptr_t to represent the numerical values of addresses.
 * */
#if __riscv_xlen == 64 || defined(__x86_64__) || defined(__aarch64__) || defined(__loongarch__)
typedef int64_t intptr_t;
typedef uint64_t uintptr_t;
#elif __riscv_xlen == 32 || defined(__i386__)
typedef int32_t intptr_t;
typedef uint32_t uintptr_t;
#endif

typedef uint8_t uint_fast8_t;
typedef uint64_t uint_fast64_t;

#if UINTPTR_MAX == UINT64_MAX
#define INT64_C(c)   c##L
#define UINT64_C(c)  c##UL
#define INTMAX_C(c)  c##L
#define UINTMAX_C(c) c##UL
#else
#define INT64_C(c)   c##LL
#define UINT64_C(c)  c##ULL
#define INTMAX_C(c)  c##LL
#define UINTMAX_C(c) c##ULL
#endif

#define SIZE_MAX UINT64_MAX

#endif // __STDINT_H__
