#ifndef __ARCH_CC_H__
#define __ARCH_CC_H__

#define LWIP_NO_INTTYPES_H 1
#define LWIP_NO_LIMITS_H   1
#define LWIP_NO_CTYPE_H    1

#define SSIZE_MAX        INT_MAX
#define LWIP_NO_UNISTD_H 1

extern void ax_print();
#define LWIP_PLATFORM_DIAG(x) ax_print();
#define LWIP_PLATFORM_ASSERT(x)

#endif /* __ARCH_CC_H__ */