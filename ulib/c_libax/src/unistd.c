#include <stdio.h>
#include <sys/types.h>
#include <unistd.h>

// TODO:
long int sysconf(int name)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
off_t lseek(int fd, off_t offset, int whence)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
unsigned int sleep(unsigned int seconds)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
pid_t getpid(void)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    printf("getpid\n");
    return -1;
}

// TODO:
int fsync(int fd)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
int close(int fd)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return -1;
}

// TODO:
int access(const char *pathname, int mode)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
char *getcwd(char *buf, size_t size)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
int lstat(const char *path, struct stat *buf)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
int stat(const char *path, struct stat *buf)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
int fstat(int fd, struct stat *buf)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
int ftruncate(int fd, off_t length)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
ssize_t read(int fd, void *buf, size_t count)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
ssize_t write(int fd, const void *buf, size_t count)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
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
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
uid_t geteuid(void)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

// TODO:
ssize_t readlink(const char *path, char *buf, size_t bufsiz)
{
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}
