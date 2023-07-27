#ifndef _POLL_H
#define _POLL_H

struct pollfd {
    int fd;
    short events;
    short revents;
};

#define POLLIN   0x001
#define POLLPRI  0x002
#define POLLOUT  0x004
#define POLLERR  0x008
#define POLLHUP  0x010
#define POLLNVAL 0x020

typedef unsigned long nfds_t;

int poll(struct pollfd *__fds, nfds_t __nfds, int __timeout);

#endif // _POLL_H
