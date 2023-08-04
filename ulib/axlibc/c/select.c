#ifdef AX_CONFIG_SELECT

#include <axlibc.h>
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/select.h>
#include <sys/time.h>

int select(int n, fd_set *__restrict rfds, fd_set *__restrict wfds, fd_set *__restrict efds,
           struct timeval *__restrict tv)
{
    time_t s = tv ? tv->tv_sec : 0;
    long us = tv ? tv->tv_usec : 0;
    // long ns;
    const time_t max_time = (1ULL << (8 * sizeof(time_t) - 1)) - 1;

    if (s < 0 || us < 0) {
        errno = EINVAL;
        return -1;
    }
    if (us / 1000000 > max_time - s) {
        s = max_time;
        us = 999999;
        // ns = 999999999;
    } else {
        s += us / 1000000;
        us %= 1000000;
        // ns = us * 1000;
    }

    return ax_select(n, rfds, wfds, efds, tv ? ((struct timeval *)(long[]){s, us}) : NULL);
}

int pselect(int n, fd_set *restrict rfds, fd_set *restrict wfds, fd_set *restrict efds,
            const struct timespec *restrict ts, const sigset_t *restrict mask)
{
    struct timeval tv = {ts->tv_sec, ts->tv_nsec / 1000};
    select(n, rfds, wfds, efds, &tv);
    return 0;
}

#endif // AX_CONFIG_SELECT
