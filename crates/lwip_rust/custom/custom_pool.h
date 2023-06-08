
#ifndef __CUSTOM_POOL_H__
#define __CUSTOM_POOL_H__

#include "lwip/pbuf.h"

typedef struct rx_custom_pbuf_t {
    struct pbuf_custom p;
    void *buf;
    void *dev;
} rx_custom_pbuf_t;

void rx_custom_pbuf_init(void);
struct pbuf *rx_custom_pbuf_alloc(pbuf_free_custom_fn custom_free_function, void *buf, void *dev,
                                  u16_t length, void *payload_mem, u16_t payload_mem_len);
void rx_custom_pbuf_free(rx_custom_pbuf_t *p);

#endif /* __CUSTOM_POOL_H__ */