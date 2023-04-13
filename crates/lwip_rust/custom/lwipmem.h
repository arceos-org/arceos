#ifndef __LWIP_MEM_H__
#define __LWIP_MEM_H__

#include <stddef.h>

void *lwip_memcpy(void *dst, const void *src, size_t len);
void *lwip_memmove(void *dst, const void *src, size_t len);

#endif /* __LWIP_MEM_H__ */
