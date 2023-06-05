#include <libax.h>
#include <stdio.h>
#include <sys/socket.h>
#include <sys/types.h>

#if defined(AX_CONFIG_NET)
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
#endif
