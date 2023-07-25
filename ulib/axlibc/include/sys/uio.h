#ifndef _SYS_UIO_H
#define _SYS_UIO_H

#include <stddef.h>

struct iovec {
    void *iov_base; /* Pointer to data.  */
    size_t iov_len; /* Length of data.  */
};

ssize_t writev(int, const struct iovec *, int);

#endif
