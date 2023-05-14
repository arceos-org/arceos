#include <libax.h>
#include <stdio.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

// TODO:
uid_t geteuid(void)
{
    unimplemented();
    return 0;
}

pid_t getpid(void)
{
#ifdef AX_CONFIG_MULTITASK
    return ax_getpid();
#else
    // return 'main' task Id
    return 2;
#endif
}

unsigned int sleep(unsigned int seconds)
{
    struct timespec ts;

    ts.tv_sec = seconds;
    ts.tv_nsec = 0;
    if (nanosleep(&ts, &ts))
        return ts.tv_sec;

    return 0;
}

int usleep(unsigned int usec)
{
    struct timespec ts;

    ts.tv_sec = (long int)(usec / 1000000);
    ts.tv_nsec = (long int)usec % 1000;
    if (nanosleep(&ts, &ts))
        return -1;

    return 0;
}

// TODO:
long int sysconf(int name)
{
    unimplemented();
    return 0;
}

int close(int fd)
{
    return ax_close(fd);
}

int fstat(int fd, struct stat *buf)
{
    return ax_fstat(fd, buf);
}

ssize_t read(int fd, void *buf, size_t count)
{
    return ax_read(fd, buf, count);
}

ssize_t write(int fd, const void *buf, size_t count)
{
    return ax_write(fd, buf, count);
}

#ifdef AX_CONFIG_FS
// TODO:
int access(const char *pathname, int mode)
{
    unimplemented();
    return 0;
}

char *getcwd(char *buf, size_t size)
{
    return ax_getcwd(buf, size);
}

int lstat(const char *path, struct stat *buf)
{
    return ax_lstat(path, buf);
}

int stat(const char *path, struct stat *buf)
{
    return ax_stat(path, buf);
}

// TODO:
ssize_t readlink(const char *path, char *buf, size_t bufsiz)
{
    unimplemented();
    return 0;
}

// TODO:
int unlink(const char *pathname)
{
    unimplemented();
    return 0;
}

// TODO:
int rmdir(const char *pathname)
{
    unimplemented();
    return 0;
}

off_t lseek(int fd, off_t offset, int whence)
{
    return ax_lseek(fd, offset, whence);
}

// TODO:
int fsync(int fd)
{
    unimplemented();
    return 0;
}

// TODO:
int fchown(int fd, uid_t owner, gid_t group)
{
    unimplemented("owner: %x group: %x", owner, group);
    return 0;
}

// TODO:
int ftruncate(int fd, off_t length)
{
    unimplemented();
    return 0;
}

#endif
