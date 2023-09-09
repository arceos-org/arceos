#ifdef AX_CONFIG_SELECT

#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/select.h>
#include <sys/time.h>

int pselect(int n, fd_set *restrict rfds, fd_set *restrict wfds, fd_set *restrict efds,
            const struct timespec *restrict ts, const sigset_t *restrict mask)
{
    struct timeval tv = {ts->tv_sec, ts->tv_nsec / 1000};
    select(n, rfds, wfds, efds, &tv);
    return 0;
}

#endif // AX_CONFIG_SELECT
