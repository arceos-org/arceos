#ifdef AX_CONFIG_NET

#include <axlibc.h>
#include <ctype.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>

#include <arpa/inet.h>
#include <netdb.h>
#include <netinet/in.h>

int h_errno;

/*Only IPv4. Ports are always 0. Ignore service and hint. Results' ai_flags, ai_socktype,
 * ai_protocol and ai_canonname are 0 or NULL.  */
int getaddrinfo(const char *__restrict node, const char *__restrict service,
                const struct addrinfo *__restrict hints, struct addrinfo **__restrict res)
{
    struct sockaddr *addrs = (struct sockaddr *)malloc(MAXADDRS * sizeof(struct sockaddr));
    int res_len = ax_getaddrinfo(node, service, addrs, MAXADDRS);
    if (res_len < 0)
        return EAI_FAIL;
    if (res_len == 0)
        return EAI_NONAME;
    struct addrinfo *_res = (struct addrinfo *)calloc(res_len, sizeof(struct addrinfo));
    for (int i = 0; i < res_len; i++) {
        (_res + i)->ai_family = AF_INET;
        (_res + i)->ai_addrlen = sizeof(struct sockaddr);
        (_res + i)->ai_addr = (addrs + i);
        (_res + i)->ai_next = (_res + i + 1);
        // TODO: This is a hard-code part, only return TCP parameters
        (_res + i)->ai_socktype = SOCK_STREAM;
        (_res + i)->ai_protocol = IPPROTO_TCP;
    }
    (_res + res_len - 1)->ai_next = NULL;
    *res = _res;
    return 0;
}

void freeaddrinfo(struct addrinfo *__restrict res)
{
    free(res->ai_addr);
    free(res);
    return;
}

static const char gai_msgs[] = "Invalid flags\0"
                               "Name does not resolve\0"
                               "Try again\0"
                               "Non-recoverable error\0"
                               "Unknown error\0"
                               "Unrecognized address family or invalid length\0"
                               "Unrecognized socket type\0"
                               "Unrecognized service\0"
                               "Unknown error\0"
                               "Out of memory\0"
                               "System error\0"
                               "Overflow\0"
                               "\0Unknown error";

const char *gai_strerror(int ecode)
{
    const char *s;
    for (s = gai_msgs, ecode++; ecode && *s; ecode++, s++)
        for (; *s; s++)
            ;
    if (!*s)
        s++;
    return s;
}

static const char msgs[] = "Host not found\0"
                           "Try again\0"
                           "Non-recoverable error\0"
                           "Address not available\0"
                           "\0Unknown error";

const char *hstrerror(int ecode)
{
    const char *s;
    for (s = msgs, ecode--; ecode && *s; ecode--, s++)
        for (; *s; s++)
            ;
    if (!*s)
        s++;
    return s;
}

static __inline uint16_t __bswap_16(uint16_t __x)
{
    return __x << 8 | __x >> 8;
}

static __inline uint32_t __bswap_32(uint32_t __x)
{
    return __x >> 24 | (__x >> 8 & 0xff00) | (__x << 8 & 0xff0000) | __x << 24;
}

uint32_t htonl(uint32_t n)
{
    union {
        int i;
        char c;
    } u = {1};
    return u.c ? __bswap_32(n) : n;
}

uint16_t htons(uint16_t n)
{
    union {
        int i;
        char c;
    } u = {1};
    return u.c ? __bswap_16(n) : n;
}

uint32_t ntohl(uint32_t n)
{
    union {
        int i;
        char c;
    } u = {1};
    return u.c ? __bswap_32(n) : n;
}

uint16_t ntohs(uint16_t n)
{
    union {
        int i;
        char c;
    } u = {1};
    return u.c ? __bswap_16(n) : n;
}

static int hexval(unsigned c)
{
    if (c - '0' < 10)
        return c - '0';
    c |= 32;
    if (c - 'a' < 6)
        return c - 'a' + 10;
    return -1;
}

int inet_pton(int af, const char *__restrict s, void *__restrict a0)
{
    uint16_t ip[8];
    unsigned char *a = a0;
    int i, j, v, d, brk = -1, need_v4 = 0;

    if (af == AF_INET) {
        for (i = 0; i < 4; i++) {
            for (v = j = 0; j < 3 && isdigit(s[j]); j++) v = 10 * v + s[j] - '0';
            if (j == 0 || (j > 1 && s[0] == '0') || v > 255)
                return 0;
            a[i] = v;
            if (s[j] == 0 && i == 3)
                return 1;
            if (s[j] != '.')
                return 0;
            s += j + 1;
        }
        return 0;
    } else if (af != AF_INET6) {
        errno = EAFNOSUPPORT;
        return -1;
    }

    if (*s == ':' && *++s != ':')
        return 0;

    for (i = 0;; i++) {
        if (s[0] == ':' && brk < 0) {
            brk = i;
            ip[i & 7] = 0;
            if (!*++s)
                break;
            if (i == 7)
                return 0;
            continue;
        }
        for (v = j = 0; j < 4 && (d = hexval(s[j])) >= 0; j++) v = 16 * v + d;
        if (j == 0)
            return 0;
        ip[i & 7] = v;
        if (!s[j] && (brk >= 0 || i == 7))
            break;
        if (i == 7)
            return 0;
        if (s[j] != ':') {
            if (s[j] != '.' || (i < 6 && brk < 0))
                return 0;
            need_v4 = 1;
            i++;
            break;
        }
        s += j + 1;
    }
    if (brk >= 0) {
        for (j = 0; j < 7 - i; j++) ip[brk + j] = 0;
        memmove(ip + brk + 7 - i, ip + brk, 2 * (i + 1 - brk));
    }
    for (j = 0; j < 8; j++) {
        *a++ = ip[j] >> 8;
        *a++ = ip[j];
    }
    if (need_v4 && inet_pton(AF_INET, (void *)s, a - 4) <= 0)
        return 0;
    return 1;
}

const char *inet_ntop(int af, const void *__restrict a0, char *__restrict s, socklen_t l)
{
    const unsigned char *a = a0;
    int i, j, max, best;
    char buf[100];

    switch (af) {
    case AF_INET:
        if (snprintf(s, l, "%d.%d.%d.%d", a[0], a[1], a[2], a[3]) < l)
            return s;
        break;
    case AF_INET6:
        if (memcmp(a, "\0\0\0\0\0\0\0\0\0\0\377\377", 12))
            snprintf(buf, sizeof buf, "%x:%x:%x:%x:%x:%x:%x:%x", 256 * a[0] + a[1],
                     256 * a[2] + a[3], 256 * a[4] + a[5], 256 * a[6] + a[7], 256 * a[8] + a[9],
                     256 * a[10] + a[11], 256 * a[12] + a[13], 256 * a[14] + a[15]);
        else
            snprintf(buf, sizeof buf, "%x:%x:%x:%x:%x:%x:%d.%d.%d.%d", 256 * a[0] + a[1],
                     256 * a[2] + a[3], 256 * a[4] + a[5], 256 * a[6] + a[7], 256 * a[8] + a[9],
                     256 * a[10] + a[11], a[12], a[13], a[14], a[15]);
        /* Replace longest /(^0|:)[:0]{2,}/ with "::" */
        for (i = best = 0, max = 2; buf[i]; i++) {
            if (i && buf[i] != ':')
                continue;
            j = strspn(buf + i, ":0");
            if (j > max)
                best = i, max = j;
        }
        if (max > 3) {
            buf[best] = buf[best + 1] = ':';
            memmove(buf + best + 2, buf + best + max, i - best - max + 1);
        }
        if (strlen(buf) < l) {
            strcpy(s, buf);
            return s;
        }
        break;
    default:
        errno = EAFNOSUPPORT;
        return 0;
    }
    errno = ENOSPC;
    return 0;
}

#endif // AX_CONFIG_NET
