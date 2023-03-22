#include <fcntl.h>
#include <stdio.h>

// TODO:
int fcntl(int fd, int cmd, ... /* arg */)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
int open(const char *filename, int flags, ...)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    printf("open file: %s\n", filename);
    return -1;
}
