#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#ifdef AX_CONFIG_ALLOC
int closedir(DIR *dir)
{
    int ret = close(dir->fd);
    free(dir);
    return ret;
}

DIR *fdopendir(int fd)
{
    DIR *dir;
    struct stat st;

    if (fstat(fd, &st) < 0) {
        return 0;
    }
    if (fcntl(fd, F_GETFL) & O_PATH) {
        errno = EBADF;
        return 0;
    }
    if (!S_ISDIR(st.st_mode)) {
        errno = ENOTDIR;
        return 0;
    }
    if (!(dir = calloc(1, sizeof(*dir)))) {
        return 0;
    }

    fcntl(fd, F_SETFD, FD_CLOEXEC);
    dir->fd = fd;
    return dir;
}

#endif

int dirfd(DIR *d)
{
    return d->fd;
}
