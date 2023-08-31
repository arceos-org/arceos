#include <axlibc.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
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

// TODO
uid_t getuid(void)
{
    unimplemented();
    return 0;
}

// TODO
pid_t setsid(void)
{
    unimplemented();
    return 0;
}

// TODO
int isatty(int fd)
{
    unimplemented();
    return 0;
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

int usleep(unsigned useconds)
{
    struct timespec tv = {.tv_sec = useconds / 1000000, .tv_nsec = (useconds % 1000000) * 1000};
    return nanosleep(&tv, &tv);
}

long sysconf(int name)
{
    return ax_sysconf(name);
}

#ifdef AX_CONFIG_FD

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

int dup(int fd)
{
    return ax_dup(fd);
}

int dup2(int old, int new)
{
    int r;
    if (old == new) {
        r = fcntl(old, F_GETFD);
        if (r >= 0)
            return old;
        else
            return r;
    }
    return ax_dup3(old, new, 0);
}

int dup3(int old, int new, int flags)
{
    return ax_dup3(old, new, flags);
}

#endif // AX_CONFIG_FD

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

// TODO
int fdatasync(int __fildes)
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

// TODO
int chdir(const char *__path)
{
    unimplemented();
    return 0;
}

// TODO
int truncate(const char *path, off_t length)
{
    unimplemented();
    return 0;
}

#endif // AX_CONFIG_FS

#ifdef AX_CONFIG_PIPE

int pipe(int fd[2])
{
    return ax_pipe(&fd[0], &fd[1]);
}

int pipe2(int fd[2], int flag)
{
    if (!flag)
        return pipe(fd);
    if (flag & ~(O_CLOEXEC | O_NONBLOCK))
        return -EINVAL;

    int res = pipe(fd);
    if (res != 0)
        return res;

    if (flag & O_CLOEXEC) {
        fcntl(fd[0], F_SETFD, FD_CLOEXEC);
        fcntl(fd[1], F_SETFD, FD_CLOEXEC);
    }
    if (flag & O_NONBLOCK) {
        fcntl(fd[0], F_SETFL, O_NONBLOCK);
        fcntl(fd[1], F_SETFL, O_NONBLOCK);
    }

    return 0;
}

#endif // AX_CONFIG_PIPE

// TODO
_Noreturn void _exit(int status)
{
    ax_exit(status);
}

// TODO
int execve(const char *__path, char *const *__argv, char *const *__envp)
{
    unimplemented();
    return 0;
}

// TODO
pid_t fork(void)
{
    unimplemented();
    return -1;
}
