#ifndef _MATH_H
#define _MATH_H

#include <stddef.h>

#define FP_NAN       0
#define FP_INFINITE  1
#define FP_ZERO      2
#define FP_SUBNORMAL 3
#define FP_NORMAL    4

#define LOG_TABLE_BITS  7
#define LOG_POLY_ORDER  6
#define LOG_POLY1_ORDER 12

#if defined(AX_CONFIG_FP_SIMD)

#if 100 * __GNUC__ + __GNUC_MINOR__ >= 303
#define NAN      __builtin_nanf("")
#define INFINITY __builtin_inff()
#else
#define NAN      (0.0f / 0.0f)
#define INFINITY 1e5000f
#endif

#ifndef fp_barrier
#define fp_barrier fp_barrier
static inline double fp_barrier(double x)
{
    volatile double y = x;
    return y;
}
#endif

#define WANT_ROUNDING 1

static inline float eval_as_float(float x)
{
    float y = x;
    return y;
}

static inline double eval_as_double(double x)
{
    double y = x;
    return y;
}

#ifndef fp_force_evalf
#define fp_force_evalf fp_force_evalf
static inline void fp_force_evalf(float x)
{
    volatile float y;
    y = x;
}
#endif

#ifndef fp_force_eval
#define fp_force_eval fp_force_eval
static inline void fp_force_eval(double x)
{
    volatile double y;
    y = x;
}
#endif

#ifndef fp_force_evall
#define fp_force_evall fp_force_evall
static inline void fp_force_evall(long double x)
{
    volatile long double y;
    y = x;
}
#endif

#ifdef __GNUC__
#define predict_true(x)  __builtin_expect(!!(x), 1)
#define predict_false(x) __builtin_expect(x, 0)
#else
#define predict_true(x)  (x)
#define predict_false(x) (x)
#endif

#define HUGE_VALF INFINITY
#define HUGE_VAL  ((double)INFINITY)
#define HUGE_VALL ((long double)INFINITY)

#define M_PI 3.14159265358979323846 /* pi */

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

#define fpclassify(x)                                 \
    (sizeof(x) == sizeof(float)    ? __fpclassifyf(x) \
     : sizeof(x) == sizeof(double) ? __fpclassify(x)  \
                                   : __fpclassifyl(x))

#define isnan(x)                                                                      \
    (sizeof(x) == sizeof(float)    ? (__FLOAT_BITS(x) & 0x7fffffff) > 0x7f800000      \
     : sizeof(x) == sizeof(double) ? (__DOUBLE_BITS(x) & -1ULL >> 1) > 0x7ffULL << 52 \
                                   : __fpclassifyl(x) == FP_NAN)

#define isinf(x)                                                                       \
    (sizeof(x) == sizeof(float)    ? (__FLOAT_BITS(x) & 0x7fffffff) == 0x7f800000      \
     : sizeof(x) == sizeof(double) ? (__DOUBLE_BITS(x) & -1ULL >> 1) == 0x7ffULL << 52 \
                                   : __fpclassifyl(x) == FP_INFINITE)

#define isfinite(x)                                                                   \
    (sizeof(x) == sizeof(float)    ? (__FLOAT_BITS(x) & 0x7fffffff) < 0x7f800000      \
     : sizeof(x) == sizeof(double) ? (__DOUBLE_BITS(x) & -1ULL >> 1) < 0x7ffULL << 52 \
                                   : __fpclassifyl(x) > FP_INFINITE)

#define isnormal(x)                                                                           \
    (sizeof(x) == sizeof(float) ? ((__FLOAT_BITS(x) + 0x00800000) & 0x7fffffff) >= 0x01000000 \
                                : ((__DOUBLE_BITS(x) + (1ULL << 52)) & -1ULL >> 1) >= 1ULL << 53)

typedef double double_t;

int __fpclassify(double);
int __fpclassifyf(float);
int __fpclassifyl(long double);

int __eqtf2(long double a, long double b);
int __gttf2(long double a, long double b);
long double __floatditf(long i);
long double __extenddftf2(double a);
long double __addtf3(long double a, long double b);
long double __multf3(long double a, long double b);
double __trunctfdf2(long double a);
long __fixtfdi(long double a);

long double roundl(long double x);
double rint(double x);
long long llrint(double);
double floor(double __x);
double sqrt(double __x);
double pow(double __x, double __y);
long long llroundl(long double __x);
double ceil(double __x);
double log(double __x);
double cos(double __x);
double fabs(double __x);
double sin(double __x);
double asin(double __x);
double round(double __x);
long double ceill(long double __x);

double copysign(double, double);
long double copysignl(long double, long double);

double acos(double);
double atan(double);
double atan2(double, double);
double cosh(double);
double exp(double);

double frexp(double, int *);
double ldexp(double, int);
double log10(double);
double modf(double, double *);
double sinh(double);
double tan(double);
double tanh(double);

double scalbn(double, int);
long double scalbnl(long double, int);

double fmod(double, double);
long double fmodl(long double, long double);

long double fabsl(long double);

double __floatunsidf(unsigned i);
long double __floatsitf(int i);
long double __subtf3(long double a, long double b);
int __getf2(long double a, long double b);
int __netf2(long double a, long double b);

long double __divtf3(long double a, long double b);
int __letf2(long double a, long double b);

uint64_t __bswapdi2(uint64_t u);
long double __floatunsitf(int i);

#endif
#endif
