#ifdef AX_CONFIG_FP_SIMD

#include <float.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>

#include "libm.h"

int __fpclassify(double x)
{
    union {
        double f;
        uint64_t i;
    } u = {x};
    int e = u.i >> 52 & 0x7ff;
    if (!e)
        return u.i << 1 ? FP_SUBNORMAL : FP_ZERO;
    if (e == 0x7ff)
        return u.i << 12 ? FP_NAN : FP_INFINITE;
    return FP_NORMAL;
}

int __fpclassifyf(float x)
{
    union {
        float f;
        uint32_t i;
    } u = {x};
    int e = u.i >> 23 & 0xff;
    if (!e)
        return u.i << 1 ? FP_SUBNORMAL : FP_ZERO;
    if (e == 0xff)
        return u.i << 9 ? FP_NAN : FP_INFINITE;
    return FP_NORMAL;
}

#if LDBL_MANT_DIG == 53 && LDBL_MAX_EXP == 1024
int __fpclassifyl(long double x)
{
    return __fpclassify(x);
}
#elif LDBL_MANT_DIG == 64 && LDBL_MAX_EXP == 16384
int __fpclassifyl(long double x)
{
    union ldshape u = {x};
    int e = u.i.se & 0x7fff;
    int msb = u.i.m >> 63;
    if (!e && !msb)
        return u.i.m ? FP_SUBNORMAL : FP_ZERO;
    if (e == 0x7fff) {
        /* The x86 variant of 80-bit extended precision only admits
         * one representation of each infinity, with the mantissa msb
         * necessarily set. The version with it clear is invalid/nan.
         * The m68k variant, however, allows either, and tooling uses
         * the version with it clear. */
        if (__BYTE_ORDER == __LITTLE_ENDIAN && !msb)
            return FP_NAN;
        return u.i.m << 1 ? FP_NAN : FP_INFINITE;
    }
    if (!msb)
        return FP_NAN;
    return FP_NORMAL;
}
#elif LDBL_MANT_DIG == 113 && LDBL_MAX_EXP == 16384
int __fpclassifyl(long double x)
{
    union ldshape u = {x};
    int e = u.i.se & 0x7fff;
    u.i.se = 0;
    if (!e)
        return u.i2.lo | u.i2.hi ? FP_SUBNORMAL : FP_ZERO;
    if (e == 0x7fff)
        return u.i2.lo | u.i2.hi ? FP_NAN : FP_INFINITE;
    return FP_NORMAL;
}
#endif

double fabs(double x)
{
    union {
        double f;
        uint64_t i;
    } u = {x};
    u.i &= -1ULL / 2;
    return u.f;
}

static const double toint = 1 / DBL_EPSILON;

double floor(double x)
{
    union {
        double f;
        uint64_t i;
    } u = {x};
    int e = u.i >> 52 & 0x7ff;
    double y;

    if (e >= 0x3ff + 52 || x == 0)
        return x;
    /* y = int(x) - x, where int(x) is an integer neighbor of x */
    if (u.i >> 63)
        y = x - toint + toint - x;
    else
        y = x + toint - toint - x;
    /* special case because of non-nearest rounding modes */
    if (e <= 0x3ff - 1) {
        FORCE_EVAL(y);
        return u.i >> 63 ? -1 : 0;
    }
    if (y > 0)
        return x + y - 1;
    return x + y;
}

double rint(double x)
{
    unimplemented();
    return 0;
}

long long llrint(double x)
{
    return rint(x);
}

double sqrt(double x)
{
    unimplemented();
    return 0;
}

double round(double x)
{
    unimplemented();
    return x;
}

long double roundl(long double x)
{
    unimplemented();
    return x;
}

long long llroundl(long double x)
{
    unimplemented();
    return x;
}

double cos(double __x)
{
    unimplemented();
    return 0;
}

double ceil(double x)
{
    union {
        double f;
        uint64_t i;
    } u = {x};
    int e = u.i >> 52 & 0x7ff;
    double_t y;

    if (e >= 0x3ff + 52 || x == 0)
        return x;
    if (u.i >> 63)
        y = x - toint + toint - x;
    else
        y = x + toint - toint - x;
    if (e <= 0x3ff - 1) {
        FORCE_EVAL(y);
        return u.i >> 63 ? -0.0 : 1;
    }
    if (y < 0)
        return x + y + 1;
    return x + y;
}

// TODO
double sin(double __x)
{
    unimplemented();
    return 0;
}

// TODO
double asin(double __x)
{
    unimplemented();
    return 0;
}

long double ceill(long double x)
{
    unimplemented();
    return x;
}

double acos(double x)
{
    unimplemented();
    return 0;
}

// TODO
double atan(double x)
{
    unimplemented();
    return 0;
}

// TODO
double atan2(double y, double x)
{
    unimplemented();
    return 0;
}

double cosh(double x)
{
    unimplemented();
    return 0;
}

// TODO
double exp(double x)
{
    unimplemented();
    return 0;
}

// TODO
double frexp(double x, int *e)
{
    unimplemented();
    return 0;
}

double ldexp(double x, int n)
{
    unimplemented();
    return 0;
}

// TODO
double log10(double x)
{
    unimplemented();
    return 0;
}

// TODO
double modf(double x, double *iptr)
{
    unimplemented();
    return 0;
}

double sinh(double x)
{
    unimplemented();
    return 0;
}

// TODO
double tan(double x)
{
    unimplemented();
    return 0;
}

// TODO
double tanh(double x)
{
    unimplemented();
    return 0;
}

double copysign(double x, double y)
{
    union {
        double f;
        uint64_t i;
    } ux = {x}, uy = {y};
    ux.i &= -1ULL / 2;
    ux.i |= uy.i & 1ULL << 63;
    return ux.f;
}

#if LDBL_MANT_DIG == 53 && LDBL_MAX_EXP == 1024
long double copysignl(long double x, long double y)
{
    return copysign(x, y);
}
#elif (LDBL_MANT_DIG == 64 || LDBL_MANT_DIG == 113) && LDBL_MAX_EXP == 16384
long double copysignl(long double x, long double y)
{
    union ldshape ux = {x}, uy = {y};
    ux.i.se &= 0x7fff;
    ux.i.se |= uy.i.se & 0x8000;
    return ux.f;
}
#endif

double scalbn(double x, int n)
{
    union {
        double f;
        uint64_t i;
    } u;
    double_t y = x;

    if (n > 1023) {
        y *= 0x1p1023;
        n -= 1023;
        if (n > 1023) {
            y *= 0x1p1023;
            n -= 1023;
            if (n > 1023)
                n = 1023;
        }
    } else if (n < -1022) {
        /* make sure final n < -53 to avoid double
           rounding in the subnormal range */
        y *= 0x1p-1022 * 0x1p53;
        n += 1022 - 53;
        if (n < -1022) {
            y *= 0x1p-1022 * 0x1p53;
            n += 1022 - 53;
            if (n < -1022)
                n = -1022;
        }
    }
    u.i = (uint64_t)(0x3ff + n) << 52;
    x = y * u.f;
    return x;
}

#if LDBL_MANT_DIG == 53 && LDBL_MAX_EXP == 1024
long double scalbnl(long double x, int n)
{
    return scalbn(x, n);
}
#elif (LDBL_MANT_DIG == 64 || LDBL_MANT_DIG == 113) && LDBL_MAX_EXP == 16384
long double scalbnl(long double x, int n)
{
    union ldshape u;

    if (n > 16383) {
        x *= 0x1p16383L;
        n -= 16383;
        if (n > 16383) {
            x *= 0x1p16383L;
            n -= 16383;
            if (n > 16383)
                n = 16383;
        }
    } else if (n < -16382) {
        x *= 0x1p-16382L * 0x1p113L;
        n += 16382 - 113;
        if (n < -16382) {
            x *= 0x1p-16382L * 0x1p113L;
            n += 16382 - 113;
            if (n < -16382)
                n = -16382;
        }
    }
    u.f = 1.0;
    u.i.se = 0x3fff + n;
    return x * u.f;
}
#endif

double fmod(double x, double y)
{
    union {
        double f;
        uint64_t i;
    } ux = {x}, uy = {y};
    int ex = ux.i >> 52 & 0x7ff;
    int ey = uy.i >> 52 & 0x7ff;
    int sx = ux.i >> 63;
    uint64_t i;

    /* in the followings uxi should be ux.i, but then gcc wrongly adds */
    /* float load/store to inner loops ruining performance and code size */
    uint64_t uxi = ux.i;

    if (uy.i << 1 == 0 || isnan(y) || ex == 0x7ff)
        return (x * y) / (x * y);
    if (uxi << 1 <= uy.i << 1) {
        if (uxi << 1 == uy.i << 1)
            return 0 * x;
        return x;
    }

    /* normalize x and y */
    if (!ex) {
        for (i = uxi << 12; i >> 63 == 0; ex--, i <<= 1)
            ;
        uxi <<= -ex + 1;
    } else {
        uxi &= -1ULL >> 12;
        uxi |= 1ULL << 52;
    }
    if (!ey) {
        for (i = uy.i << 12; i >> 63 == 0; ey--, i <<= 1)
            ;
        uy.i <<= -ey + 1;
    } else {
        uy.i &= -1ULL >> 12;
        uy.i |= 1ULL << 52;
    }

    /* x mod y */
    for (; ex > ey; ex--) {
        i = uxi - uy.i;
        if (i >> 63 == 0) {
            if (i == 0)
                return 0 * x;
            uxi = i;
        }
        uxi <<= 1;
    }
    i = uxi - uy.i;
    if (i >> 63 == 0) {
        if (i == 0)
            return 0 * x;
        uxi = i;
    }
    for (; uxi >> 52 == 0; uxi <<= 1, ex--)
        ;

    /* scale result */
    if (ex > 0) {
        uxi -= 1ULL << 52;
        uxi |= (uint64_t)ex << 52;
    } else {
        uxi >>= -ex + 1;
    }
    uxi |= (uint64_t)sx << 63;
    ux.i = uxi;
    return ux.f;
}

// x86_64 has specific implementation
#if LDBL_MANT_DIG == 53 && LDBL_MAX_EXP == 1024
long double fmodl(long double x, long double y)
{
    return fmod(x, y);
}
#elif (LDBL_MANT_DIG == 64 || LDBL_MANT_DIG == 113) && LDBL_MAX_EXP == 16384
long double fmodl(long double x, long double y)
{
    union ldshape ux = {x}, uy = {y};
    int ex = ux.i.se & 0x7fff;
    int ey = uy.i.se & 0x7fff;
    int sx = ux.i.se & 0x8000;

    if (y == 0 || isnan(y) || ex == 0x7fff)
        return (x * y) / (x * y);
    ux.i.se = ex;
    uy.i.se = ey;
    if (ux.f <= uy.f) {
        if (ux.f == uy.f)
            return 0 * x;
        return x;
    }

    /* normalize x and y */
    if (!ex) {
        ux.f *= 0x1p120f;
        ex = ux.i.se - 120;
    }
    if (!ey) {
        uy.f *= 0x1p120f;
        ey = uy.i.se - 120;
    }

    /* x mod y */
#if LDBL_MANT_DIG == 64
    uint64_t i, mx, my;
    mx = ux.i.m;
    my = uy.i.m;
    for (; ex > ey; ex--) {
        i = mx - my;
        if (mx >= my) {
            if (i == 0)
                return 0 * x;
            mx = 2 * i;
        } else if (2 * mx < mx) {
            mx = 2 * mx - my;
        } else {
            mx = 2 * mx;
        }
    }
    i = mx - my;
    if (mx >= my) {
        if (i == 0)
            return 0 * x;
        mx = i;
    }
    for (; mx >> 63 == 0; mx *= 2, ex--)
        ;
    ux.i.m = mx;
#elif LDBL_MANT_DIG == 113
    uint64_t hi, lo, xhi, xlo, yhi, ylo;
    xhi = (ux.i2.hi & -1ULL >> 16) | 1ULL << 48;
    yhi = (uy.i2.hi & -1ULL >> 16) | 1ULL << 48;
    xlo = ux.i2.lo;
    ylo = uy.i2.lo;
    for (; ex > ey; ex--) {
        hi = xhi - yhi;
        lo = xlo - ylo;
        if (xlo < ylo)
            hi -= 1;
        if (hi >> 63 == 0) {
            if ((hi | lo) == 0)
                return 0 * x;
            xhi = 2 * hi + (lo >> 63);
            xlo = 2 * lo;
        } else {
            xhi = 2 * xhi + (xlo >> 63);
            xlo = 2 * xlo;
        }
    }
    hi = xhi - yhi;
    lo = xlo - ylo;
    if (xlo < ylo)
        hi -= 1;
    if (hi >> 63 == 0) {
        if ((hi | lo) == 0)
            return 0 * x;
        xhi = hi;
        xlo = lo;
    }
    for (; xhi >> 48 == 0; xhi = 2 * xhi + (xlo >> 63), xlo = 2 * xlo, ex--)
        ;
    ux.i2.hi = xhi;
    ux.i2.lo = xlo;
#endif

    /* scale result */
    if (ex <= 0) {
        ux.i.se = (ex + 120) | sx;
        ux.f *= 0x1p-120f;
    } else
        ux.i.se = ex | sx;
    return ux.f;
}
#endif

#if LDBL_MANT_DIG == 53 && LDBL_MAX_EXP == 1024
long double fabsl(long double x)
{
    return fabs(x);
}
#elif (LDBL_MANT_DIG == 64 || LDBL_MANT_DIG == 113) && LDBL_MAX_EXP == 16384
long double fabsl(long double x)
{
    union ldshape u = {x};

    u.i.se &= 0x7fff;
    return u.f;
}
#endif

#endif // AX_CONFIG_FP_SIMD
