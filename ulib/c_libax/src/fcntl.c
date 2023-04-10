#include <fcntl.h>
#include <libax.h>
#include <stdio.h>

// TODO:
int fcntl(int fd, int cmd, ... /* arg */)
{
    printf("%s%s fd: %d cmd: %d\n", "Error: no ax_call implementation for ", __func__, fd, cmd);
    return 0;
}

int open(const char *filename, int flags, ...)
{
    return ax_open(filename, flags);
}
