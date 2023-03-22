#include <stdio.h>
#include <sys/time.h>

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
