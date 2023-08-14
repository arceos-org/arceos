#ifndef __SOCKET_H__
#define __SOCKET_H__

#include <stddef.h>

#ifndef SOCK_STREAM
#define SOCK_STREAM 1
#define SOCK_DGRAM  2
#endif

#define SOCK_RAW       3
#define SOCK_RDM       4
#define SOCK_SEQPACKET 5
#define SOCK_DCCP      6
#define SOCK_PACKET    10

#ifndef SOCK_CLOEXEC
#define SOCK_CLOEXEC  02000000
#define SOCK_NONBLOCK 04000
#endif

#define PF_UNSPEC     0
#define PF_LOCAL      1
#define PF_UNIX       PF_LOCAL
#define PF_FILE       PF_LOCAL
#define PF_INET       2
#define PF_AX25       3
#define PF_IPX        4
#define PF_APPLETALK  5
#define PF_NETROM     6
#define PF_BRIDGE     7
#define PF_ATMPVC     8
#define PF_X25        9
#define PF_INET6      10
#define PF_ROSE       11
#define PF_DECnet     12
#define PF_NETBEUI    13
#define PF_SECURITY   14
#define PF_KEY        15
#define PF_NETLINK    16
#define PF_ROUTE      PF_NETLINK
#define PF_PACKET     17
#define PF_ASH        18
#define PF_ECONET     19
#define PF_ATMSVC     20
#define PF_RDS        21
#define PF_SNA        22
#define PF_IRDA       23
#define PF_PPPOX      24
#define PF_WANPIPE    25
#define PF_LLC        26
#define PF_IB         27
#define PF_MPLS       28
#define PF_CAN        29
#define PF_TIPC       30
#define PF_BLUETOOTH  31
#define PF_IUCV       32
#define PF_RXRPC      33
#define PF_ISDN       34
#define PF_PHONET     35
#define PF_IEEE802154 36
#define PF_CAIF       37
#define PF_ALG        38
#define PF_NFC        39
#define PF_VSOCK      40
#define PF_KCM        41
#define PF_QIPCRTR    42
#define PF_SMC        43
#define PF_XDP        44
#define PF_MAX        45

#define AF_UNSPEC     PF_UNSPEC
#define AF_LOCAL      PF_LOCAL
#define AF_UNIX       AF_LOCAL
#define AF_FILE       AF_LOCAL
#define AF_INET       PF_INET
#define AF_AX25       PF_AX25
#define AF_IPX        PF_IPX
#define AF_APPLETALK  PF_APPLETALK
#define AF_NETROM     PF_NETROM
#define AF_BRIDGE     PF_BRIDGE
#define AF_ATMPVC     PF_ATMPVC
#define AF_X25        PF_X25
#define AF_INET6      PF_INET6
#define AF_ROSE       PF_ROSE
#define AF_DECnet     PF_DECnet
#define AF_NETBEUI    PF_NETBEUI
#define AF_SECURITY   PF_SECURITY
#define AF_KEY        PF_KEY
#define AF_NETLINK    PF_NETLINK
#define AF_ROUTE      PF_ROUTE
#define AF_PACKET     PF_PACKET
#define AF_ASH        PF_ASH
#define AF_ECONET     PF_ECONET
#define AF_ATMSVC     PF_ATMSVC
#define AF_RDS        PF_RDS
#define AF_SNA        PF_SNA
#define AF_IRDA       PF_IRDA
#define AF_PPPOX      PF_PPPOX
#define AF_WANPIPE    PF_WANPIPE
#define AF_LLC        PF_LLC
#define AF_IB         PF_IB
#define AF_MPLS       PF_MPLS
#define AF_CAN        PF_CAN
#define AF_TIPC       PF_TIPC
#define AF_BLUETOOTH  PF_BLUETOOTH
#define AF_IUCV       PF_IUCV
#define AF_RXRPC      PF_RXRPC
#define AF_ISDN       PF_ISDN
#define AF_PHONET     PF_PHONET
#define AF_IEEE802154 PF_IEEE802154
#define AF_CAIF       PF_CAIF
#define AF_ALG        PF_ALG
#define AF_NFC        PF_NFC
#define AF_VSOCK      PF_VSOCK
#define AF_KCM        PF_KCM
#define AF_QIPCRTR    PF_QIPCRTR
#define AF_SMC        PF_SMC
#define AF_XDP        PF_XDP
#define AF_MAX        PF_MAX

#define SO_DEBUG       1
#define SO_REUSEADDR   2
#define SO_TYPE        3
#define SO_ERROR       4
#define SO_DONTROUTE   5
#define SO_BROADCAST   6
#define SO_SNDBUF      7
#define SO_RCVBUF      8
#define SO_KEEPALIVE   9
#define SO_OOBINLINE   10
#define SO_NO_CHECK    11
#define SO_PRIORITY    12
#define SO_LINGER      13
#define SO_BSDCOMPAT   14
#define SO_REUSEPORT   15
#define SO_PASSCRED    16
#define SO_PEERCRED    17
#define SO_RCVLOWAT    18
#define SO_SNDLOWAT    19
#define SO_ACCEPTCONN  30
#define SO_PEERSEC     31
#define SO_SNDBUFFORCE 32
#define SO_RCVBUFFORCE 33
#define SO_PROTOCOL    38
#define SO_DOMAIN      39

#define SOL_SOCKET 1

#define SOL_IP     0
#define SOL_IPV6   41
#define SOL_ICMPV6 58

#define SOL_RAW       255
#define SOL_DECNET    261
#define SOL_X25       262
#define SOL_PACKET    263
#define SOL_ATM       264
#define SOL_AAL       265
#define SOL_IRDA      266
#define SOL_NETBEUI   267
#define SOL_LLC       268
#define SOL_DCCP      269
#define SOL_NETLINK   270
#define SOL_TIPC      271
#define SOL_RXRPC     272
#define SOL_PPPOL2TP  273
#define SOL_BLUETOOTH 274
#define SOL_PNPIPE    275
#define SOL_RDS       276
#define SOL_IUCV      277
#define SOL_CAIF      278
#define SOL_ALG       279
#define SOL_NFC       280
#define SOL_KCM       281
#define SOL_TLS       282
#define SOL_XDP       283

typedef unsigned short sa_family_t;
struct sockaddr {
    sa_family_t sa_family;
    char sa_data[14];
};

struct sockaddr_storage {
    sa_family_t ss_family;
    char __ss_padding[128 - sizeof(long) - sizeof(sa_family_t)];
    unsigned long __ss_align;
};

typedef unsigned socklen_t;

#if defined(AX_CONFIG_NET)
int socket(int, int, int);
int shutdown(int, int);

int bind(int, const struct sockaddr *, socklen_t);
int connect(int, const struct sockaddr *, socklen_t);
int listen(int, int);
int accept(int, struct sockaddr *__restrict, socklen_t *__restrict);

ssize_t send(int, const void *, size_t, int);
ssize_t recv(int, void *, size_t, int);
ssize_t sendto(int, const void *, size_t, int, const struct sockaddr *, socklen_t);
ssize_t recvfrom(int, void *__restrict, size_t, int, struct sockaddr *__restrict,
                 socklen_t *__restrict);

int getsockopt(int, int, int, void *__restrict, socklen_t *__restrict);
int setsockopt(int, int, int, const void *, socklen_t);

int getsockname(int sockfd, struct sockaddr *restrict addr, socklen_t *restrict addrlen);
int getpeername(int sockfd, struct sockaddr *restrict addr, socklen_t *restrict addrlen);

#endif
#endif
