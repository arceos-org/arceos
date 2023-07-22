#include <ctype.h>
#include <errno.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include <axlibc.h>

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

int strcoll(const char *l, const char *r)
{
    return strcmp(l, r);
}

#define BITOP(a, b, op) \
    ((a)[(size_t)(b) / (8 * sizeof *(a))] op(size_t) 1 << ((size_t)(b) % (8 * sizeof *(a))))
#define MAX(a, b) ((a) > (b) ? (a) : (b))
#define MIN(a, b) ((a) < (b) ? (a) : (b))

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

size_t strspn(const char *s, const char *c)
{
    const char *a = s;
    size_t byteset[32 / sizeof(size_t)] = {0};

    if (!c[0])
        return 0;
    if (!c[1]) {
        for (; *s == *c; s++)
            ;
        return s - a;
    }

    for (; *c && BITOP(byteset, *(unsigned char *)c, |=); c++)
        ;
    for (; *s && BITOP(byteset, *(unsigned char *)s, &); s++)
        ;
    return s - a;
}

char *strpbrk(const char *s, const char *b)
{
    s += strcspn(s, b);
    return *s ? (char *)s : 0;
}

char *strchrnul(const char *s, int c)
{
    c = (unsigned char)c;
    if (!c)
        return (char *)s + strlen(s);

    for (; *s && *(unsigned char *)s != c; s++)
        ;
    return (char *)s;
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

char *strerror(int e)
{
    return ax_errno_string(e);
}

int strerror_r(int err, char *buf, size_t buflen)
{
    char *msg = strerror(err);
    size_t l = strlen(msg);
    if (l >= buflen) {
        if (buflen) {
            memcpy(buf, msg, buflen - 1);
            buf[buflen - 1] = 0;
        }
        return ERANGE;
    }
    memcpy(buf, msg, l + 1);
    return 0;
}

void *memcpy(void *restrict dest, const void *restrict src, size_t n)
{
    unsigned char *d = dest;
    const unsigned char *s = src;
    for (; n; n--) *d++ = *s++;
    return dest;
}

void *memmove(void *dest, const void *src, size_t n)
{
    char *d = dest;
    const char *s = src;

    if (d == s)
        return d;
    if ((uintptr_t)s - (uintptr_t)d - n <= -2 * n)
        return memcpy(d, s, n);

    if (d < s) {
        for (; n; n--) *d++ = *s++;
    } else {
        while (n) n--, d[n] = s[n];
    }

    return dest;
}

int memcmp(const void *vl, const void *vr, size_t n)
{
    const unsigned char *l = vl, *r = vr;
    for (; n && *l == *r; n--, l++, r++)
        ;
    return n ? *l - *r : 0;
}

int strcasecmp(const char *_l, const char *_r)
{
    const unsigned char *l = (void *)_l, *r = (void *)_r;
    for (; *l && *r && (*l == *r || tolower(*l) == tolower(*r)); l++, r++)
        ;
    return tolower(*l) - tolower(*r);
}

int strncasecmp(const char *_l, const char *_r, size_t n)
{
    const unsigned char *l = (void *)_l, *r = (void *)_r;
    if (!n--)
        return 0;
    for (; *l && *r && n && (*l == *r || tolower(*l) == tolower(*r)); l++, r++, n--)
        ;
    return tolower(*l) - tolower(*r);
}

// `strstr` helper function
static char *twobyte_strstr(const unsigned char *h, const unsigned char *n)
{
    uint16_t nw = n[0] << 8 | n[1], hw = h[0] << 8 | h[1];
    for (h++; *h && hw != nw; hw = hw << 8 | *++h)
        ;
    return *h ? (char *)h - 1 : 0;
}

// `strstr` helper function
static char *threebyte_strstr(const unsigned char *h, const unsigned char *n)
{
    uint32_t nw = (uint32_t)n[0] << 24 | n[1] << 16 | n[2] << 8;
    uint32_t hw = (uint32_t)h[0] << 24 | h[1] << 16 | h[2] << 8;
    for (h += 2; *h && hw != nw; hw = (hw | *++h) << 8)
        ;
    return *h ? (char *)h - 2 : 0;
}

// `strstr` helper function
static char *fourbyte_strstr(const unsigned char *h, const unsigned char *n)
{
    uint32_t nw = (uint32_t)n[0] << 24 | n[1] << 16 | n[2] << 8 | n[3];
    uint32_t hw = (uint32_t)h[0] << 24 | h[1] << 16 | h[2] << 8 | h[3];
    for (h += 3; *h && hw != nw; hw = hw << 8 | *++h)
        ;
    return *h ? (char *)h - 3 : 0;
}

// `strstr` helper function
static char *twoway_strstr(const unsigned char *h, const unsigned char *n)
{
    const unsigned char *z;
    size_t l, ip, jp, k, p, ms, p0, mem, mem0;
    size_t byteset[32 / sizeof(size_t)] = {0};
    size_t shift[256];

    /* Computing length of needle and fill shift table */
    for (l = 0; n[l] && h[l]; l++) BITOP(byteset, n[l], |=), shift[n[l]] = l + 1;
    if (n[l])
        return 0; /* hit the end of h */

    /* Compute maximal suffix */
    ip = -1;
    jp = 0;
    k = p = 1;
    while (jp + k < l) {
        if (n[ip + k] == n[jp + k]) {
            if (k == p) {
                jp += p;
                k = 1;
            } else
                k++;
        } else if (n[ip + k] > n[jp + k]) {
            jp += k;
            k = 1;
            p = jp - ip;
        } else {
            ip = jp++;
            k = p = 1;
        }
    }
    ms = ip;
    p0 = p;

    /* And with the opposite comparison */
    ip = -1;
    jp = 0;
    k = p = 1;
    while (jp + k < l) {
        if (n[ip + k] == n[jp + k]) {
            if (k == p) {
                jp += p;
                k = 1;
            } else
                k++;
        } else if (n[ip + k] < n[jp + k]) {
            jp += k;
            k = 1;
            p = jp - ip;
        } else {
            ip = jp++;
            k = p = 1;
        }
    }
    if (ip + 1 > ms + 1)
        ms = ip;
    else
        p = p0;

    /* Periodic needle? */
    if (memcmp(n, n + p, ms + 1)) {
        mem0 = 0;
        p = MAX(ms, l - ms - 1) + 1;
    } else
        mem0 = l - p;
    mem = 0;

    /* Initialize incremental end-of-haystack pointer */
    z = h;

    /* Search loop */
    for (;;) {
        /* Update incremental end-of-haystack pointer */
        if (z - h < l) {
            /* Fast estimate for MAX(l,63) */
            size_t grow = l | 63;
            const unsigned char *z2 = memchr(z, 0, grow);
            if (z2) {
                z = z2;
                if (z - h < l)
                    return 0;
            } else
                z += grow;
        }

        /* Check last byte first; advance by shift on mismatch */
        if (BITOP(byteset, h[l - 1], &)) {
            k = l - shift[h[l - 1]];
            if (k) {
                if (k < mem)
                    k = mem;
                h += k;
                mem = 0;
                continue;
            }
        } else {
            h += l;
            mem = 0;
            continue;
        }

        /* Compare right half */
        for (k = MAX(ms + 1, mem); n[k] && n[k] == h[k]; k++)
            ;
        if (n[k]) {
            h += k - ms;
            mem = 0;
            continue;
        }
        /* Compare left half */
        for (k = ms + 1; k > mem && n[k - 1] == h[k - 1]; k--)
            ;
        if (k <= mem)
            return (char *)h;
        h += p;
        mem = mem0;
    }
}

char *strstr(const char *h, const char *n)
{
    /* Return immediately on empty needle */
    if (!n[0])
        return (char *)h;

    /* Use faster algorithms for short needles */
    h = strchr(h, *n);
    if (!h || !n[1])
        return (char *)h;
    if (!h[1])
        return 0;
    if (!n[2])
        return twobyte_strstr((void *)h, (void *)n);
    if (!h[2])
        return 0;
    if (!n[3])
        return threebyte_strstr((void *)h, (void *)n);
    if (!h[3])
        return 0;
    if (!n[4])
        return fourbyte_strstr((void *)h, (void *)n);

    return twoway_strstr((void *)h, (void *)n);
}

#ifdef AX_CONFIG_ALLOC

#include <stdlib.h>
char *strdup(const char *s)
{
    size_t l = strlen(s);
    char *d = malloc(l + 1);
    if (!d)
        return NULL;
    return memcpy(d, s, l + 1);
}

#endif // AX_CONFIG_ALLOC
