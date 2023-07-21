#include <stdio.h>
#include <sys/stat.h>
#include <sys/types.h>

// TODO:
int fchmod(int fd, mode_t mode)
{
    unimplemented();
    return 0;
}

// TODO:
int mkdir(const char *path, mode_t mode)
{
    unimplemented();
    return 0;
}

// TODO
int chmod(const char *path, mode_t mode)
{
    unimplemented();
    return 0;
}

// TODO
mode_t umask(mode_t mask)
{
    unimplemented("mask: %d", mask);
    return 0;
}

// TODO
int fstatat(int fd, const char *restrict path, struct stat *restrict st, int flag)
{
    unimplemented();
    return 0;
}
