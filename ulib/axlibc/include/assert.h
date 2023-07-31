#ifndef __ASSERT_H__
#define __ASSERT_H__

#include <features.h>

#if __STDC_VERSION__ >= 201112L && !defined(__cplusplus)
#define static_assert _Static_assert
#endif

#define assert(x) ((void)((x) || (__assert_fail(#x, __FILE__, __LINE__, __func__), 0)))

_Noreturn void __assert_fail(const char *, const char *, int, const char *);

#endif // __ASSERT_H__
