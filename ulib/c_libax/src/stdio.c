#include <assert.h>
#include <fcntl.h>
#include <limits.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include <libax.h>

#define MAX(a, b) ((a) > (b) ? (a) : (b))
#define MIN(a, b) ((a) < (b) ? (a) : (b))

#define __LINE_WIDTH 256

FILE __stdin_FILE = {
    .fd = 0,
};

FILE __stdout_FILE = {
    .fd = 1,
};

FILE __stderr_FILE = {
    .fd = 2,
};

FILE *const stdin = &__stdin_FILE;
FILE *const stdout = &__stdout_FILE;
FILE *const stderr = &__stderr_FILE;

static char buffer[__LINE_WIDTH];
static int buffer_len;

// Returns: number of chars written, negative for failure
// Warn: buffer_len[f] will not be changed
static int __write_buffer()
{
    if (buffer_len == 0)
        return 0;
    int r = ax_print_str(buffer, buffer_len);
    return r;
}

// Clear buffer_len[f]
static void __clear_buffer()
{
    buffer_len = 0;
}

static int __fflush()
{
    int r = __write_buffer();
    __clear_buffer();
    return r >= 0 ? 0 : r;
}

static char digits[] = "0123456789abcdef";

static int out(int f, const char *s, size_t l)
{
    int ret = 0;
    for (size_t i = 0; i < l; i++) {
        char c = s[i];
        buffer[buffer_len++] = c;
        if (buffer_len == __LINE_WIDTH || c == '\n') {
            int r = __write_buffer();
            __clear_buffer();
            if (r < 0)
                return r;
            if (r < buffer_len)
                return ret + r;
            ret += r;
        }
    }
    return ret;
}

static void printint(long long value, int base, int sign)
{
    const int buf_size = 20;
    char buf[buf_size + 1];
    int i;
    uint64_t x;

    if (sign && (sign = (value < 0)))
        x = -value;
    else
        x = value;

    buf[buf_size] = 0;
    i = buf_size;

    do {
        buf[--i] = digits[x % base];
    } while ((x /= base) != 0);

    if (sign)
        buf[--i] = '-';
    assert(i >= 0);
    out(stdout->fd, buf + i, buf_size - i);
}

static void printptr(uint64_t value)
{
    int i = 0, j;
    char buf[32 + 1];
    buf[i++] = '0';
    buf[i++] = 'x';
    for (j = 0; j < (sizeof(uint64_t) * 2); j++, value <<= 4)
        buf[i++] = digits[value >> (sizeof(uint64_t) * 8 - 4)];
    buf[i] = 0;
    out(stdout->fd, buf, i);
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
    if (f->fd == 1 || f->fd == 2)
        return __fflush();
    return 0;
}

int putchar(int c)
{
    char byte = c;
    return out(stdout->fd, &byte, 1);
}

int puts(const char *s)
{
    return ax_println_str(s, strlen(s));
}

// Print to the file. only understands %d, %x, %p, %s.
void fprintf(int f, const char *restrict fmt, ...)
{
    va_list ap;
    int l = 0;
    char *a, *z, *s = (char *)fmt;

    va_start(ap, fmt);
    for (;;) {
        if (!*s)
            break;
        for (a = s; *s && *s != '%'; s++)
            ;
        for (z = s; s[0] == '%' && s[1] == '%'; z++, s += 2)
            ;
        l = z - a;
        out(f, a, l);
        if (l)
            continue;
        if (s[1] == 0)
            break;
        switch (s[1]) {
        case 'u':
            printint(va_arg(ap, int), 10, 0);
            break;
        case 'c':
            putchar((char)va_arg(ap, int));
            break;
        case 'd':
            printint(va_arg(ap, int), 10, 1);
            break;
        case 'x':
            printint(va_arg(ap, int), 16, 1);
            break;
        case 'p':
            printptr(va_arg(ap, uint64_t));
            break;
        case 's':
            if ((a = va_arg(ap, char *)) == 0)
                a = "(null)";
            l = strnlen(a, 500);
            out(f, a, l);
            break;
        case 'l':
            if (s[2] == 'u')
                printint(va_arg(ap, long), 10, 0);
            else if (s[2] == 'd')
                printint(va_arg(ap, long), 10, 1);
            else if (s[2] == 'x')
                printint(va_arg(ap, long), 16, 1);
            else {
                putchar('%');
                putchar(s[1]);
                if (s[2])
                    putchar(s[2]);
                else
                    s -= 1;
            }
            s += 1;
            break;
        default:
            // Print unknown % sequence to draw attention.
            putchar('%');
            putchar(s[1]);
            break;
        }
        s += 2;
    }
    va_end(ap);
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

    // TODO: Should set errno
    if (!strchr("rwa", *mode)) {
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

#endif

// Helper function for vsnprintf()
static void parsenint(char *dst, int *n, size_t m, long long value, int base, int sign)
{
    const int buf_size = 20;
    char buf[buf_size + 1];
    int i;
    uint64_t x;

    if (sign && (sign = (value < 0)))
        x = -value;
    else
        x = value;

    i = buf_size;

    do {
        buf[i--] = digits[x % base];
    } while ((x /= base) != 0);

    if (sign)
        buf[i--] = '-';
    assert(i >= 0);
    for (int j = i + 1; j <= buf_size && *n < m; j++) {
        dst[(*n)++] = buf[j];
    }
}

// Helper function for vsnprintf()
static void parsenptr(char *dst, int *n, size_t m, uint64_t value)
{
    int i = 0, j;
    char buf[32 + 1];
    buf[i++] = '0';
    buf[i++] = 'x';
    for (j = 0; j < (sizeof(uint64_t) * 2); j++, value <<= 4)
        buf[i++] = digits[value >> (sizeof(uint64_t) * 8 - 4)];
    for (int k = 0; k < i && *n < m; k++) {
        dst[(*n)++] = buf[k];
    }
}

int vsnprintf(char *restrict buf, size_t size, const char *restrict fmt, va_list ap)
{
    int l = 0;
    char *a, *z, *s = (char *)fmt;
    int cnt = 0;
    int n = size - 1;

    for (; cnt < n;) {
        if (!*s)
            break;
        while (*s && *s != '%') {
            buf[cnt++] = *s;
            s++;
            if (cnt == n)
                break;
        }
        if (cnt == n || !*s)
            break;
        for (z = s; s[0] == '%' && s[1] == '%'; z++, s += 2) {
            buf[cnt++] = '%';
            if (cnt == n)
                break;
        }
        if (cnt == n || !*s)
            break;
        if (*s != '%')
            continue;
        if (s[1] == 0)
            break;
        switch (s[1]) {
        case 'u':
            parsenint(buf, &cnt, n, va_arg(ap, int), 10, 0);
            break;
        case 'c':
            buf[cnt++] = (char)va_arg(ap, int);
            break;
        case 'd':
            parsenint(buf, &cnt, n, va_arg(ap, int), 10, 1);
            break;
        case 'x':
            parsenint(buf, &cnt, n, va_arg(ap, int), 16, 1);
            break;
        case 'p':
            parsenptr(buf, &cnt, n, va_arg(ap, uint64_t));
            break;
        case 's':
            if ((a = va_arg(ap, char *)) == 0)
                a = "(null)";
            l = strnlen(a, 200);
            for (int i = 0; i < l && cnt < n; i++) {
                buf[cnt++] = a[i];
            }
            break;
        case 'l':
            if (s[2] == 'u') {
                parsenint(buf, &cnt, n, va_arg(ap, long), 10, 0);
            } else if (s[2] == 'd') {
                parsenint(buf, &cnt, n, va_arg(ap, int), 10, 1);
            } else if (s[2] == 'x') {
                parsenint(buf, &cnt, n, va_arg(ap, int), 16, 1);
            } else if (s[2] == 'l' && s[3] == 'u') {
                parsenint(buf, &cnt, n, va_arg(ap, long long), 10, 0);
                s += 1;
            } else {
                buf[cnt++] = '%';
                if (cnt == n)
                    break;
                buf[cnt++] = s[1];
                if (cnt == n)
                    break;
                if (s[2]) {
                    buf[cnt++] = s[2];
                    if (cnt == n)
                        break;
                } else {
                    s -= 1;
                }
            }
            s += 1;
            break;
        default:
            buf[cnt++] = '%';
            if (cnt == n)
                break;
            buf[cnt++] = s[1];
            if (cnt == n)
                break;
            break;
        }
        s += 2;
    }
    buf[cnt] = '\0';
    return cnt;
}

int snprintf(char *restrict s, size_t n, const char *restrict fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    int ret = vsnprintf(s, n, fmt, ap);
    va_end(ap);
    return ret;
}

int vsprintf(char *restrict s, const char *restrict fmt, va_list ap)
{
    return vsnprintf(s, INT_MAX, fmt, ap);
}

/// Currently print formatted text to stdout or stderr.
/// TODO: print to file
int vfprintf(FILE *restrict f, const char *restrict fmt, va_list ap)
{
    int ret;
    char buf[1024];

    ret = vsnprintf(buf, sizeof(buf), fmt, ap);
    if (ret < 0)
        return ret;

    if (f->fd == stdout->fd || f->fd == stderr->fd)
        out(f->fd, buf, sizeof(buf));

    return ret;
}

int sprintf(char *restrict s, const char *restrict fmt, ...)
{
    int ret;
    va_list ap;
    va_start(ap, fmt);
    ret = vsprintf(s, fmt, ap);
    va_end(ap);
    return ret;
}
