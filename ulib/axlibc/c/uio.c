#include <stdio.h>
#include <sys/uio.h>

#include <axlibc.h>

ssize_t writev(int fd, const struct iovec *iovec, int count)
{
    return ax_writev(fd, iovec, count);
}
