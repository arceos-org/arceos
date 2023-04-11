#ifndef __ARCH_CC_H__
#define __ARCH_CC_H__

extern void ax_print();
#define LWIP_PLATFORM_DIAG(x) ax_print();

#endif /* __ARCH_CC_H__ */