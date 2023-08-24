#include <fcntl.h>
#include <stdarg.h>
#include <stdio.h>

#ifdef AX_CONFIG_FD

// TODO: remove this function in future work
int ax_fcntl(int fd, int cmd, size_t arg);

int fcntl(int fd, int cmd, ... /* arg */)
{
    unsigned long arg;
    va_list ap;
    va_start(ap, cmd);
    arg = va_arg(ap, unsigned long);
    va_end(ap);

    return ax_fcntl(fd, cmd, arg);
}

#endif // AX_CONFIG_FD

#ifdef AX_CONFIG_FS

// TODO: remove this function in future work
int ax_open(const char *filename, int flags, mode_t mode);

int open(const char *filename, int flags, ...)
{
    mode_t mode = 0;

    if ((flags & O_CREAT) || (flags & O_TMPFILE) == O_TMPFILE) {
        va_list ap;
        va_start(ap, flags);
        mode = va_arg(ap, mode_t);
        va_end(ap);
    }

    return ax_open(filename, flags, mode);
}

// TODO
int posix_fadvise(int __fd, unsigned long __offset, unsigned long __len, int __advise)
{
    unimplemented();
    return 0;
}

// TODO
int sync_file_range(int fd, off_t pos, off_t len, unsigned flags)
{
    unimplemented();
    return 0;
}

#endif // AX_CONFIG_FS
