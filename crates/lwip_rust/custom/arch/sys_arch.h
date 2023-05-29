#ifndef __ARCH_SYS_ARCH_H__
#define __ARCH_SYS_ARCH_H__

#define SYS_MBOX_NULL NULL
#define SYS_SEM_NULL  NULL

#define isspace(a) ((a == ' ' || (unsigned)a - '\t' < 5))
#define isdigit(a) (((unsigned)(a) - '0') < 10)

int strcmp(const char *l, const char *r);

#endif /* __ARCH_SYS_ARCH_H__ */