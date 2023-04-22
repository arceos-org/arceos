#include <assert.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include <libax.h>

#define MAX(a, b) ((a) > (b) ? (a) : (b))
#define MIN(a, b) ((a) < (b) ? (a) : (b))

#define __LINE_WIDTH 256

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
    out(stdout, buf + i, buf_size - i);
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
    out(stdout, buf, i);
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

int fflush(int fd)
{
    if (fd == stdout || fd == stderr)
        return __fflush();
    return 0;
}

int putchar(int c)
{
    char byte = c;
    return out(stdout, &byte, 1);
}

int puts(const char *s)
{
    int r;
    r = -(out(stdout, s, strlen(s)) < 0 || putchar('\n') < 0);
    return r;
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
            l = strnlen(a, 200);
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
