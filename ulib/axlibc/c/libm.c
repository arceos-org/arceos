#ifdef AX_CONFIG_FP_SIMD

#include <math.h>

#include "libm.h"

double __math_divzero(uint32_t sign)
{
    return fp_barrier(sign ? -1.0 : 1.0) / 0.0;
}

double __math_invalid(double x)
{
    return (x - x) / (x - x);
}

#endif // AX_CONFIG_FP_SIMD
