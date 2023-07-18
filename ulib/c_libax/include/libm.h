#ifndef _LIBM_H
#define _LIBM_H

#include <endian.h>
#include <float.h>
#include <stdint.h>

#if defined(AX_CONFIG_FP_SIMD)

#define asuint(f)    \
    ((union {        \
        float _f;    \
        uint32_t _i; \
    }){f})           \
        ._i
#define asfloat(i)   \
    ((union {        \
        uint32_t _i; \
        float _f;    \
    }){i})           \
        ._f
#define asuint64(f)  \
    ((union {        \
        double _f;   \
        uint64_t _i; \
    }){f})           \
        ._i
#define asdouble(i)  \
    ((union {        \
        uint64_t _i; \
        double _f;   \
    }){i})           \
        ._f

#if LDBL_MANT_DIG == 53 && LDBL_MAX_EXP == 1024
#elif LDBL_MANT_DIG == 64 && LDBL_MAX_EXP == 16384 && __BYTE_ORDER == __LITTLE_ENDIAN
union ldshape {
    long double f;
    struct {
        uint64_t m;
        uint16_t se;
    } i;
};
#elif LDBL_MANT_DIG == 64 && LDBL_MAX_EXP == 16384 && __BYTE_ORDER == __BIG_ENDIAN
/* This is the m68k variant of 80-bit long double, and this definition only works
 * on archs where the alignment requirement of uint64_t is <= 4. */
union ldshape {
    long double f;
    struct {
        uint16_t se;
        uint16_t pad;
        uint64_t m;
    } i;
};
#elif LDBL_MANT_DIG == 113 && LDBL_MAX_EXP == 16384 && __BYTE_ORDER == __LITTLE_ENDIAN
union ldshape {
    long double f;
    struct {
        uint64_t lo;
        uint32_t mid;
        uint16_t top;
        uint16_t se;
    } i;
    struct {
        uint64_t lo;
        uint64_t hi;
    } i2;
};
#elif LDBL_MANT_DIG == 113 && LDBL_MAX_EXP == 16384 && __BYTE_ORDER == __BIG_ENDIAN
union ldshape {
    long double f;
    struct {
        uint16_t se;
        uint16_t top;
        uint32_t mid;
        uint64_t lo;
    } i;
    struct {
        uint64_t hi;
        uint64_t lo;
    } i2;
};
#else
#error Unsupported long double representation
#endif

double __math_divzero(uint32_t);
double __math_invalid(double);

#endif

#endif
