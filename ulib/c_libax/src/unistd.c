#include <libax.h>
#include <stdio.h>
#include <sys/types.h>
#include <unistd.h>

// TODO:
uid_t geteuid(void)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
pid_t getpid(void)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return -1;
}

// TODO:
unsigned int sleep(unsigned int seconds)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
long int sysconf(int name)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

#ifdef AX_CONFIG_FS

off_t lseek(int fd, off_t offset, int whence)
{
    return ax_lseek(fd, offset, whence);
}

// TODO:
int fsync(int fd)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int close(int fd)
{
    return ax_close(fd);
}

// TODO:
int access(const char *pathname, int mode)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

char *getcwd(char *buf, size_t size)
{
    return ax_getcwd(buf, size);
}

// TODO:
int lstat(const char *path, struct stat *buf)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int stat(const char *path, struct stat *buf)
{
    return ax_stat(path, buf);
}

int fstat(int fd, struct stat *buf)
{
    return ax_fstat(fd, buf);
}

// TODO:
int ftruncate(int fd, off_t length)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

ssize_t read(int fd, void *buf, size_t count)
{
    return ax_read(fd, buf, count);
}

ssize_t write(int fd, const void *buf, size_t count)
{
    return ax_write(fd, buf, count);
}

// TODO:
int unlink(const char *pathname)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
int rmdir(const char *pathname)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
int fchown(int fd, uid_t owner, gid_t group)
{
    printf("%s%s owner: %x group: %x\n", "Error: no ax_call implementation for ", __func__, owner,
           group);
    return 0;
}

// TODO:
ssize_t readlink(const char *path, char *buf, size_t bufsiz)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

#endif
