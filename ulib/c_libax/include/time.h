#ifndef __TIME_H__
#define __TIME_H__

#include <stddef.h>
#include <sys/time.h>

typedef long time_t;

#define CLOCK_REALTIME  0
#define CLOCK_MONOTONIC 1
#define CLOCKS_PER_SEC  1000000L

struct tm {
    int tm_sec;   /* seconds of minute */
    int tm_min;   /* minutes of hour */
    int tm_hour;  /* hours of day */
    int tm_mday;  /* day of month */
    int tm_mon;   /* month of year, 0 is first month(January) */
    int tm_year;  /* years, whose value equals the actual year minus 1900 */
    int tm_wday;  /* day of week, 0 is sunday, 1 is monday, and so on */
    int tm_yday;  /* day of year */
    int tm_isdst; /* daylight saving time flag */
    long int __tm_gmtoff;
    const char *__tm_zone;
};

size_t strftime(char *__restrict__ _Buf, size_t _SizeInBytes, const char *__restrict__ _Format,
                const struct tm *__restrict__ _Tm);

struct tm *gmtime(const time_t *timer);

struct tm *localtime(const time_t *timep);
time_t time(time_t *t);
int clock_gettime(clockid_t _clk, struct timespec *ts);
int nanosleep(const struct timespec *requested_time, struct timespec *remaining);

#ifdef AX_CONFIG_FP_SIMD
double difftime(time_t, time_t);
#endif

#endif
