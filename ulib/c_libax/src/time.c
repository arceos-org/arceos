#include <stddef.h>
#include <stdio.h>
#include <sys/time.h>
#include <sys/types.h>
#include <time.h>

// TODO:
size_t strftime(char *__restrict__ _Buf, size_t _SizeInBytes, const char *__restrict__ _Format,
                const struct tm *__restrict__ _Tm)
{
    return 0;
}

// TODO:
struct tm *gmtime(const time_t *timer)
{
    return NULL;
}

// TODO:
struct tm *localtime(const time_t *timep)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
time_t time(time_t *t)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
int gettimeofday(struct timeval *tv, struct timezone *tz)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
int utimes(const char *filename, const struct timeval times[2])
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}
