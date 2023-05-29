#include "arch/sys_arch.h"
#include "lwip/opt.h"
#include "lwip/sys.h"

int strcmp(const char *l, const char *r)
{
    for (; *l == *r && *l; l++, r++)
        ;
    return *(unsigned char *)l - *(unsigned char *)r;
}