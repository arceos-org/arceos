#ifndef __SYS_TIME_H__
#define __SYS_TIME_H__

struct timeval {
    long tv_sec;  /* seconds */
    long tv_usec; /* microseconds */
};

struct timezone {
    int tz_minuteswest; /* (minutes west of Greenwich) */
    int tz_dsttime;     /* (type of DST correction) */
};

typedef long timezone;
int gettimeofday(struct timeval *tv, struct timezone *tz);
int utimes(const char *filename, const struct timeval times[2]);

#endif
