#ifndef _SYS_SELECT_H
#define _SYS_SELECT_H

#include <signal.h>
#include <stddef.h>
#include <sys/time.h>

#define FD_SETSIZE 1024

typedef unsigned long fd_mask;

typedef struct {
    unsigned long fds_bits[FD_SETSIZE / 8 / sizeof(long)];
} fd_set;

#define FD_ZERO(s)                                                        \
    do {                                                                  \
        int __i;                                                          \
        unsigned long *__b = (s)->fds_bits;                               \
        for (__i = sizeof(fd_set) / sizeof(long); __i; __i--) *__b++ = 0; \
    } while (0)
#define FD_SET(d, s) \
    ((s)->fds_bits[(d) / (8 * sizeof(long))] |= (1UL << ((d) % (8 * sizeof(long)))))
#define FD_CLR(d, s) \
    ((s)->fds_bits[(d) / (8 * sizeof(long))] &= ~(1UL << ((d) % (8 * sizeof(long)))))
#define FD_ISSET(d, s) \
    !!((s)->fds_bits[(d) / (8 * sizeof(long))] & (1UL << ((d) % (8 * sizeof(long)))))

int select(int n, fd_set *__restrict rfds, fd_set *__restrict wfds, fd_set *__restrict efds,
           struct timeval *__restrict tv);
int pselect(int, fd_set *__restrict, fd_set *__restrict, fd_set *__restrict,
            const struct timespec *__restrict, const sigset_t *__restrict);

#define NFDBITS (8 * (int)sizeof(long))

#endif
