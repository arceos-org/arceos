#ifndef __SYS_TIME_H__
#define __SYS_TIME_H__
#include <stdint.h>

/// <div rustbindgen replaces="TimeVal"></div>
struct timeval {
    uint64_t tv_sec;  /* seconds */
    uint64_t tv_usec; /* microseconds */
};

/// <div rustbindgen replaces="TimeSepc"></div>
struct timespec {
    uint64_t tv_sec;  /* seconds */
    uint64_t tv_nsec; /* nanoseconds */
};

struct timezone {
    int tz_minuteswest; /* (minutes west of Greenwich) */
    int tz_dsttime;     /* (type of DST correction) */
};

typedef long timezone;
int gettimeofday(struct timeval *tv, struct timezone *tz);
int utimes(const char *filename, const struct timeval times[2]);

#endif
