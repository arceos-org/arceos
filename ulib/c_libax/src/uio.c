#include <stdio.h>
#include <sys/uio.h>

#include <libax.h>

ssize_t writev(int fd, const struct iovec *iovec, int count)
{
    return ax_writev(fd, iovec, count);
}
