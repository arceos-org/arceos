#include <fcntl.h>
#include <libax.h>
#include <stdarg.h>
#include <stdio.h>

int fcntl(int fd, int cmd, ... /* arg */)
{
    unsigned long arg;
    va_list ap;
    va_start(ap, cmd);
    arg = va_arg(ap, unsigned long);
    va_end(ap);

    return ax_fcntl(fd, cmd, arg);
}

#ifdef AX_CONFIG_FS
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
#endif

// TODO
int posix_fadvise(int __fd, unsigned long __offset, unsigned long __len, int __advise)
{
    unimplemented();
    return 0;
}

// TODO
int sync_file_range(int, off_t, off_t, unsigned)
{
    unimplemented();
    return 0;
}
