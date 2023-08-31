#ifdef AX_CONFIG_NET

#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <sys/socket.h>
#include <sys/types.h>

#include <axlibc.h>

int socket(int domain, int type, int protocol)
{
    return ax_socket(domain, type, protocol);
}

int shutdown(int fd, int flag)
{
    return ax_shutdown(fd, flag);
}

int bind(int fd, const struct sockaddr *addr, socklen_t len)
{
    return ax_bind(fd, addr, len);
}

int connect(int fd, const struct sockaddr *addr, socklen_t len)
{
    return ax_connect(fd, addr, len);
}

int listen(int fd, int backlog)
{
    return ax_listen(fd, backlog);
}

int accept(int fd, struct sockaddr *restrict addr, socklen_t *restrict len)
{
    return ax_accept(fd, addr, len);
}

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

ssize_t send(int fd, const void *buf, size_t n, int flags)
{
    return ax_send(fd, buf, n, flags);
}

ssize_t recv(int fd, void *buf, size_t n, int flags)
{
    return ax_recv(fd, buf, n, flags);
}

ssize_t sendto(int fd, const void *buf, size_t n, int flags, const struct sockaddr *addr,
               socklen_t addr_len)
{
    if (addr == NULL && addr_len == 0)
        return ax_send(fd, buf, n, flags);
    else
        return ax_sendto(fd, buf, n, flags, addr, addr_len);
}

ssize_t recvfrom(int fd, void *restrict buf, size_t n, int flags, struct sockaddr *restrict addr,
                 socklen_t *restrict addr_len)
{
    if (addr == NULL)
        return ax_recv(fd, buf, n, flags);
    else
        return ax_recvfrom(fd, buf, n, flags, addr, addr_len);
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

int getsockname(int sockfd, struct sockaddr *restrict addr, socklen_t *restrict addrlen)
{
    return ax_getsockname(sockfd, addr, addrlen);
}

int getpeername(int sockfd, struct sockaddr *restrict addr, socklen_t *restrict addrlen)
{
    return ax_getpeername(sockfd, addr, addrlen);
}

// TODO
ssize_t sendmsg(int fd, const struct msghdr *msg, int flags)
{
    unimplemented();
    return 0;
}

#endif // AX_CONFIG_NET
