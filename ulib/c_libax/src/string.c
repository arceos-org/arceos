#include <stddef.h>
#include <stdint.h>
#include <string.h>

size_t strlen(const char *s)
{
    const char *a = s;
    for (; *s; s++)
        ;
    return s - a;
}

size_t strnlen(const char *s, size_t n)
{
    const char *p = memchr(s, 0, n);
    return p ? p - s : n;
}

int isspace(int c)
{
    return c == ' ' || (unsigned)c - '\t' < 5;
}

int isdigit(int c)
{
    return (unsigned)c - '0' < 10;
}

int atoi(const char *s)
{
    int n = 0, neg = 0;
    while (isspace(*s)) s++;
    switch (*s) {
    case '-':
        neg = 1;
    case '+':
        s++;
    }
    /* Compute n as a negative number to avoid overflow on INT_MIN */
    while (isdigit(*s)) n = 10 * n - (*s++ - '0');
    return neg ? n : -n;
}

void *memchr(const void *src, int c, size_t n)
{
    const unsigned char *s = src;
    c = (unsigned char)c;
    for (; n && *s != c; s++, n--)
        ;
    return n ? (void *)s : 0;
}

void *memset(void *dest, int c, size_t n)
{
    unsigned char *s = dest;
    size_t k;

    /* Fill head and tail with minimal branching. Each
     * conditional ensures that all the subsequently used
     * offsets are well-defined and in the dest region. */

    if (!n)
        return dest;
    s[0] = c;
    s[n - 1] = c;
    if (n <= 2)
        return dest;
    s[1] = c;
    s[2] = c;
    s[n - 2] = c;
    s[n - 3] = c;
    if (n <= 6)
        return dest;
    s[3] = c;
    s[n - 4] = c;
    if (n <= 8)
        return dest;

    /* Advance pointer to align it at a 4-byte boundary,
     * and truncate n to a multiple of 4. The previous code
     * already took care of any head/tail that get cut off
     * by the alignment. */

    k = -(uintptr_t)s & 3;
    s += k;
    n -= k;
    n &= -4;

    /* Pure C fallback with no aliasing violations. */
    for (; n; n--, s++) *s = c;

    return dest;
}

char *strcpy(char *restrict d, const char *restrict s)
{
    for (; (*d = *s); s++, d++)
        ;
    return d;
}

char *strncpy(char *restrict d, const char *restrict s, size_t n)
{
    for (; n && (*d = *s); n--, s++, d++)
        ;
    return d;
}

char *strcat(char *restrict d, const char *restrict s)
{
    strcpy(d + strlen(d), s);
    return d;
}

char *strncat(char *restrict d, const char *restrict s, size_t n)
{
    char *a = d;
    d += strlen(d);
    while (n && *s) n--, *d++ = *s++;
    *d++ = 0;
    return a;
}

int strcmp(const char *l, const char *r)
{
    for (; *l == *r && *l; l++, r++)
        ;
    return *(unsigned char *)l - *(unsigned char *)r;
}

int strncmp(const char *_l, const char *_r, size_t n)
{
    const unsigned char *l = (void *)_l, *r = (void *)_r;
    if (!n--)
        return 0;
    for (; *l && *r && n && *l == *r; l++, r++, n--)
        ;
    return *l - *r;
}

#define BITOP(a, b, op) a[(size_t)b / (8 * sizeof(size_t))] op 1 << (size_t)b % (8 * sizeof(size_t))
size_t strcspn(const char *s1, const char *s2)
{
    const char *a = s1;
    size_t byteset[32 / sizeof(size_t)];

    if (!s2[0] || !s2[1]) {
        for (; *s1 != *s2; s1++) return s1 - a;
    }
    memset(byteset, 0, sizeof byteset);

    for (; *s2 != '\0'; s2++) BITOP(byteset, *(unsigned char *)s2, |=);
    for (; *s1 && !(BITOP(byteset, *(unsigned char *)s1, &)); s1++)
        ;

    return s1 - a;
}

char *strchr(const char *s, int c)
{
    while (*s != c && *s != '\0') s++;

    if (*s == c) {
        return (char *)s;
    } else {
        return NULL;
    }
}

char *strrchr(const char *s, int c)
{
    char *isCharFind = NULL;
    if (s != NULL) {
        do {
            if (*s == (char)c) {
                isCharFind = (char *)s;
            }
        } while (*s++);
    }
    return isCharFind;
}

// TODO:
char *strerror(int n)
{
    return "";
}
