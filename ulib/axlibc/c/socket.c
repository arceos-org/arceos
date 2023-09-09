#ifdef AX_CONFIG_NET

#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <sys/socket.h>
#include <sys/types.h>

int accept4(int fd, struct sockaddr *restrict addr, socklen_t *restrict len, int flg)
{
    if (!flg)
        return accept(fd, addr, len);
    if (flg & ~(SOCK_CLOEXEC | SOCK_NONBLOCK)) {
        errno = EINVAL;
        return -1;
    }
    int ret = accept(fd, addr, len);
    if (ret < 0)
        return ret;
    if (flg & SOCK_CLOEXEC)
        fcntl(ret, F_SETFD, FD_CLOEXEC);
    if (flg & SOCK_NONBLOCK)
        fcntl(ret, F_SETFL, O_NONBLOCK);
    return ret;
}

int getsockopt(int fd, int level, int optname, void *restrict optval, socklen_t *restrict optlen)
{
    unimplemented();
    return -1;
}

int setsockopt(int fd, int level, int optname, const void *optval, socklen_t optlen)
{
    unimplemented("fd: %d, level: %d, optname: %d, optval: %d, optlen: %d", fd, level, optname,
                  *(int *)optval, optlen);
    return 0;
}

// TODO
ssize_t sendmsg(int fd, const struct msghdr *msg, int flags)
{
    unimplemented();
    return 0;
}

#endif // AX_CONFIG_NET
