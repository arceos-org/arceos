#ifndef _MATH_H
#define _MATH_H

#define FP_NAN       0
#define FP_INFINITE  1
#define FP_ZERO      2
#define FP_SUBNORMAL 3
#define FP_NORMAL    4
#if defined(AX_CONFIG_FP_SIMD)
int __fpclassify(double);
int __fpclassifyf(float);
static __inline unsigned __FLOAT_BITS(float __f)
{
    union {
        float __f;
        unsigned __i;
    } __u;
    __u.__f = __f;
    return __u.__i;
}
static __inline unsigned long long __DOUBLE_BITS(double __f)
{
    union {
        double __f;
        unsigned long long __i;
    } __u;
    __u.__f = __f;
    return __u.__i;
}

#define fpclassify(x) (sizeof(x) == sizeof(float) ? __fpclassifyf(x) : __fpclassify(x))

#define isnormal(x)                                                                           \
    (sizeof(x) == sizeof(float) ? ((__FLOAT_BITS(x) + 0x00800000) & 0x7fffffff) >= 0x01000000 \
                                : ((__DOUBLE_BITS(x) + (1ULL << 52)) & -1ULL >> 1) >= 1ULL << 53)

double fabs(double);
double floor(double);
#endif
#endif