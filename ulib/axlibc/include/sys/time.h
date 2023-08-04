#ifndef __SYS_TIME_H__
#define __SYS_TIME_H__

#include <stdint.h>

#define ITIMER_REAL    0
#define ITIMER_VIRTUAL 1
#define ITIMER_PROF    2

extern long timezone;
typedef long long time_t;

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

struct itimerval {
    struct timeval it_interval;
    struct timeval it_value;
};

int gettimeofday(struct timeval *tv, struct timezone *tz);

int getitimer(int, struct itimerval *);
int setitimer(int, const struct itimerval *__restrict, struct itimerval *__restrict);
int utimes(const char *filename, const struct timeval times[2]);

#endif
