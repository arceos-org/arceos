#include <stdio.h>
#include <sys/epoll.h>

#include <libax.h>

int epoll_create(int size)
{
    return ax_epoll_create(size);
}

int epoll_ctl(int fd, int op, int fd2, struct epoll_event *ev)
{
    return ax_epoll_ctl(fd, op, fd2, ev);
}

int epoll_wait(int fd, struct epoll_event *ev, int cnt, int to)
{
    return ax_epoll_wait(fd, ev, cnt, to);
}
