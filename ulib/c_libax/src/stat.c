#include <stdio.h>
#include <sys/stat.h>
#include <sys/types.h>

#include <libax.h>

// TODO:
int fchmod(int fd, mode_t mode)
{
    unimplemented();
    return 0;
}

// TODO:
int mkdir(const char *pathname, mode_t mode)
{
    unimplemented();
    return 0;
}

// TODO
int chmod(const char *__file, mode_t __mode)
{
    unimplemented();
    return 0;
}

// TODO
mode_t umask(mode_t mask)
{
    return ax_umask(mask);
}

// TODO
int fstatat(int, const char *__restrict, struct stat *__restrict, int)
{
    unimplemented();
    return 0;
}
