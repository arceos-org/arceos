#ifdef AX_CONFIG_FS

#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

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

int dirfd(DIR *d)
{
    return d->fd;
}

// TODO
DIR *opendir(const char *__name)
{
    unimplemented();
    return NULL;
}

// TODO
struct dirent *readdir(DIR *__dirp)
{
    unimplemented();
    return NULL;
}

// TODO
int readdir_r(DIR *restrict dir, struct dirent *restrict buf, struct dirent **restrict result)
{
    struct dirent *de;
    int errno_save = errno;
    int ret;

    // LOCK(dir->lock);
    errno = 0;
    de = readdir(dir);
    if ((ret = errno)) {
        // UNLOCK(dir->lock);
        return ret;
    }
    errno = errno_save;
    if (de)
        memcpy(buf, de, de->d_reclen);
    else
        buf = NULL;

    // UNLOCK(dir->lock);
    *result = buf;
    return 0;
}

// TODO
void rewinddir(DIR *dir)
{
    // LOCK(dir->lock);
    lseek(dir->fd, 0, SEEK_SET);
    dir->buf_pos = dir->buf_end = 0;
    dir->tell = 0;
    // UNLOCK(dir->lock);
}

#endif // AX_CONFIG_FS
