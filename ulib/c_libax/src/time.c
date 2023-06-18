#include <errno.h>
#include <limits.h>
#include <stddef.h>
#include <stdio.h>
#include <sys/time.h>
#include <time.h>

#include <libax.h>

const int SEC_PER_MIN = 60;
const int SEC_PER_HOUR = 3600;
const int MIN_PER_HOUR = 60;
const int HOUR_PER_DAY = 24;

/* 2000-03-01 (mod 400 year, immediately after feb29 */
#define LEAPOCH       (946684800LL + 86400 * (31 + 29))
#define DAYS_PER_400Y (365 * 400 + 97)
#define DAYS_PER_100Y (365 * 100 + 24)
#define DAYS_PER_4Y   (365 * 4 + 1)

// TODO:
size_t strftime(char *__restrict__ _Buf, size_t _SizeInBytes, const char *__restrict__ _Format,
                const struct tm *__restrict__ _Tm)
{
    unimplemented();
    return 0;
}

int __secs_to_tm(long long t, struct tm *tm)
{
    long long days, secs, years;
    int remdays, remsecs, remyears;
    int qc_cycles, c_cycles, q_cycles;
    int months;
    int wday, yday, leap;
    static const char days_in_month[] = {31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29};

    /* Reject time_t values whose year would overflow int */
    if (t < INT_MIN * 31622400LL || t > INT_MAX * 31622400LL)
        return -1;

    secs = t - LEAPOCH;
    days = secs / 86400;
    remsecs = secs % 86400;
    if (remsecs < 0) {
        remsecs += 86400;
        days--;
    }

    wday = (3 + days) % 7;
    if (wday < 0)
        wday += 7;

    qc_cycles = days / DAYS_PER_400Y;
    remdays = days % DAYS_PER_400Y;
    if (remdays < 0) {
        remdays += DAYS_PER_400Y;
        qc_cycles--;
    }

    c_cycles = remdays / DAYS_PER_100Y;
    if (c_cycles == 4)
        c_cycles--;
    remdays -= c_cycles * DAYS_PER_100Y;

    q_cycles = remdays / DAYS_PER_4Y;
    if (q_cycles == 25)
        q_cycles--;
    remdays -= q_cycles * DAYS_PER_4Y;

    remyears = remdays / 365;
    if (remyears == 4)
        remyears--;
    remdays -= remyears * 365;

    leap = !remyears && (q_cycles || !c_cycles);
    yday = remdays + 31 + 28 + leap;
    if (yday >= 365 + leap)
        yday -= 365 + leap;

    years = remyears + 4 * q_cycles + 100 * c_cycles + 400LL * qc_cycles;

    for (months = 0; days_in_month[months] <= remdays; months++) remdays -= days_in_month[months];

    if (months >= 10) {
        months -= 12;
        years++;
    }

    if (years + 100 > INT_MAX || years + 100 < INT_MIN)
        return -1;

    tm->tm_year = years + 100;
    tm->tm_mon = months + 2;
    tm->tm_mday = remdays + 1;
    tm->tm_wday = wday;
    tm->tm_yday = yday;

    tm->tm_hour = remsecs / 3600;
    tm->tm_min = remsecs / 60 % 60;
    tm->tm_sec = remsecs % 60;

    return 0;
}

struct tm *__gmtime_r(const time_t *restrict t, struct tm *restrict tm)
{
    if (__secs_to_tm(*t, tm) < 0) {
        errno = EOVERFLOW;
        return 0;
    }
    tm->tm_isdst = 0;
    tm->__tm_gmtoff = 0;
    // TODO: set timezone
    // tm->__tm_zone = __utc;
    return tm;
}

struct tm *gmtime(const time_t *timer)
{
    static struct tm tm;
    return __gmtime_r(timer, &tm);
}

// TODO: more field should be added
struct tm *localtime_r(const time_t *restrict t, struct tm *restrict tm)
{
    time_t sec = *t;
    tm->tm_sec = sec % SEC_PER_MIN;
    tm->tm_min = (sec / SEC_PER_MIN) % MIN_PER_HOUR;
    tm->tm_hour = (sec / SEC_PER_HOUR) % HOUR_PER_DAY;

    return tm;
}

struct tm *localtime(const time_t *timep)
{
    static struct tm tm;
    return localtime_r(timep, &tm);
}

time_t time(time_t *t)
{
    struct timespec ts;
    ax_clock_gettime(&ts);
    time_t ret = ts.tv_sec;
    if (t)
        *t = ret;
    return ret;
}

int gettimeofday(struct timeval *tv, struct timezone *tz)
{
    struct timespec ts;
    if (!tv)
        return 0;
    clock_gettime(CLOCK_REALTIME, &ts);
    tv->tv_sec = ts.tv_sec;
    tv->tv_usec = (int)ts.tv_nsec / 1000;
    return 0;
}

// TODO:
int utimes(const char *filename, const struct timeval times[2])
{
    unimplemented();
    return 0;
}

// TODO: Should match _clk,
int clock_gettime(clockid_t _clk, struct timespec *ts)
{
    return ax_clock_gettime(ts);
}

int nanosleep(const struct timespec *req, struct timespec *rem)
{
    return ax_nanosleep(req, rem);
}

#ifdef AX_CONFIG_FP_SIMD
double difftime(time_t t1, time_t t0)
{
    return t1 - t0;
}
#endif
