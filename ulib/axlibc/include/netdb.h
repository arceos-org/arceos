#ifndef _NETDB_H
#define _NETDB_H

#ifdef AX_CONFIG_NET

#include <netinet/in.h>

struct addrinfo {
    int ai_flags;
    int ai_family;
    int ai_socktype;
    int ai_protocol;
    socklen_t ai_addrlen;
    struct sockaddr *ai_addr;
    char *ai_canonname;
    struct addrinfo *ai_next;
};

struct aibuf {
    struct addrinfo ai;
    union sa {
        struct sockaddr_in sin;
        struct sockaddr_in6 sin6;
    } sa;
    volatile int lock[1];
    short slot, ref;
};

struct service {
    uint16_t port;
    unsigned char proto, socktype;
};

struct address {
    int family;
    unsigned scopeid;
    uint8_t addr[16];
    int sortkey;
};

#define AI_PASSIVE     0x01
#define AI_CANONNAME   0x02
#define AI_NUMERICHOST 0x04
#define AI_V4MAPPED    0x08
#define AI_ALL         0x10
#define AI_ADDRCONFIG  0x20
#define AI_NUMERICSERV 0x400

#define NI_NUMERICHOST  0x01
#define NI_NUMERICSERV  0x02
#define NI_NOFQDN       0x04
#define NI_NAMEREQD     0x08
#define NI_DGRAM        0x10
#define NI_NUMERICSCOPE 0x100

#define EAI_BADFLAGS -1
#define EAI_NONAME   -2
#define EAI_AGAIN    -3
#define EAI_FAIL     -4
#define EAI_FAMILY   -6
#define EAI_SOCKTYPE -7
#define EAI_SERVICE  -8
#define EAI_MEMORY   -10
#define EAI_SYSTEM   -11
#define EAI_OVERFLOW -12

#define HOST_NOT_FOUND 1
#define TRY_AGAIN      2
#define NO_RECOVERY    3
#define NO_DATA        4
#define NO_ADDRESS     NO_DATA

extern int h_errno;
const char *hstrerror(int ecode);

#define MAXSERVS 2
#define MAXADDRS 48

int getaddrinfo(const char *__restrict, const char *__restrict, const struct addrinfo *__restrict,
                struct addrinfo **__restrict);
void freeaddrinfo(struct addrinfo *);
const char *gai_strerror(int __ecode);

#endif // AX_CONFIG_NET

#endif // _NETDB_H
