#include "printf.h"
#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include <libax.h>

#define MAX(a, b) ((a) > (b) ? (a) : (b))
#define MIN(a, b) ((a) < (b) ? (a) : (b))

FILE __stdin_FILE = {.fd = 0, .buffer_len = 0};

FILE __stdout_FILE = {.fd = 1, .buffer_len = 0};

FILE __stderr_FILE = {.fd = 2, .buffer_len = 0};

FILE *const stdin = &__stdin_FILE;
FILE *const stdout = &__stdout_FILE;
FILE *const stderr = &__stderr_FILE;

// Returns: number of chars written, negative for failure
// Warn: buffer_len[f] will not be changed
static int __write_buffer(FILE *f)
{
    int r = 0;
    if (f->buffer_len == 0)
        return 0;
    if (f->fd == stdout->fd || f->fd == stderr->fd) {
        r = ax_print_str(f->buf, f->buffer_len);
#ifdef AX_CONFIG_ALLOC
    } else {
        r = write(f->fd, f->buf, f->buffer_len);
#endif
    }
    return r;
}

// Clear buffer_len[f]
static void __clear_buffer(FILE *f)
{
    f->buffer_len = 0;
}

static int __fflush(FILE *f)
{
    int r = __write_buffer(f);
    __clear_buffer(f);
    return r >= 0 ? 0 : r;
}

static int out(FILE *f, const char *s, size_t l)
{
    int ret = 0;
    for (size_t i = 0; i < l; i++) {
        char c = s[i];
        f->buf[f->buffer_len++] = c;
        if (f->buffer_len == FILE_BUF_SIZE || c == '\n') {
            int r = __write_buffer(f);
            __clear_buffer(f);
            if (r < 0)
                return r;
            if (r < f->buffer_len)
                return ret + r;
            ret += r;
        }
    }
    return ret;
}

// int getchar()
// {
//     char byte = 0;
//     if (1 == read(stdin, &byte, 1)) {
//         return byte;
//     } else {
//         return EOF;
//     }
// }

int fflush(FILE *f)
{
    return __fflush(f);
}

static inline int do_putc(int c, FILE *f)
{
    char byte = c;
    return out(f, &byte, 1);
}

int fputc(int c, FILE *f)
{
    return do_putc(c, f);
}

int putc(int c, FILE *f)
{
    return do_putc(c, f);
}

int putchar(int c)
{
    return do_putc(c, stdout);
}

int puts(const char *s)
{
    return ax_println_str(s, strlen(s)); // TODO: lock
}

void perror(const char *msg)
{
    FILE *f = stderr;
    char *errstr = strerror(errno);

    if (msg && *msg) {
        out(f, msg, strlen(msg));
        out(f, ": ", 2);
    }
    out(f, errstr, strlen(errstr));
    out(f, "\n", 1);
}

static void __out_wrapper(char c, void *arg)
{
    out(arg, &c, 1);
}

int printf(const char *restrict fmt, ...)
{
    int ret;
    va_list ap;
    va_start(ap, fmt);
    ret = vfprintf(stdout, fmt, ap);
    va_end(ap);
    return ret;
}

int fprintf(FILE *restrict f, const char *restrict fmt, ...)
{
    int ret;
    va_list ap;
    va_start(ap, fmt);
    ret = vfprintf(f, fmt, ap);
    va_end(ap);
    return ret;
}

int vfprintf(FILE *restrict f, const char *restrict fmt, va_list ap)
{
    return vfctprintf(__out_wrapper, f, fmt, ap);
}

#if defined(AX_CONFIG_ALLOC) && defined(AX_CONFIG_FS)

int __fmodeflags(const char *mode)
{
    int flags;
    if (strchr(mode, '+'))
        flags = O_RDWR;
    else if (*mode == 'r')
        flags = O_RDONLY;
    else
        flags = O_WRONLY;
    if (strchr(mode, 'x'))
        flags |= O_EXCL;
    if (strchr(mode, 'e'))
        flags |= O_CLOEXEC;
    if (*mode != 'r')
        flags |= O_CREAT;
    if (*mode == 'w')
        flags |= O_TRUNC;
    if (*mode == 'a')
        flags |= O_APPEND;
    return flags;
}

FILE *fopen(const char *filename, const char *mode)
{
    FILE *f;
    int flags;

    if (!strchr("rwa", *mode)) {
        errno = EINVAL;
        return 0;
    }

    f = (FILE *)malloc(sizeof(FILE));

    flags = __fmodeflags(mode);
    // TODO: currently mode is unused in ax_open
    int fd = ax_open(filename, flags, 0666);
    if (fd < 0)
        return NULL;
    f->fd = fd;

    return f;
}

char *fgets(char *restrict s, int n, FILE *restrict f)
{
    if (n == 0)
        return NULL;
    if (n == 1) {
        *s = '\0';
        return s;
    }

    int cnt = 0;
    while (cnt < n - 1) {
        char c;
        if (ax_read(f->fd, (void *)&c, 1) > 0) {
            if (c != '\n')
                s[cnt++] = c;
            else
                break;
        } else
            break;
    }
    s[cnt] = '\0';
    return s;
}

#endif
