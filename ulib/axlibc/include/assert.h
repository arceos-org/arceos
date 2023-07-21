#ifndef __ASSERT_H__
#define __ASSERT_H__

#define assert(x) ((void)((x) || (__assert_fail(#x, __FILE__, __LINE__, __func__), 0)))

_Noreturn void __assert_fail(const char *, const char *, int, const char *);

#endif // __ASSERT_H__
