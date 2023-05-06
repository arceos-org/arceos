#ifndef __SYS_TIME_H__
#define __SYS_TIME_H__

#include <stdint.h>

typedef long time_t;

struct timeval {
    time_t tv_sec; /* seconds */
    long tv_usec;  /* microseconds */
};

struct timespec {
    time_t tv_sec; /* seconds */
    long tv_nsec;  /* nanoseconds */
};

typedef struct timespec timespec;

struct timezone {
    int tz_minuteswest; /* (minutes west of Greenwich) */
    int tz_dsttime;     /* (type of DST correction) */
};

int gettimeofday(struct timeval *tv, struct timezone *tz);
int utimes(const char *filename, const struct timeval times[2]);

#endif
