#ifndef _NETINET_IN_H
#define _NETINET_IN_H

#include <sys/socket.h>

#define INET_ADDRSTRLEN  16
#define INET6_ADDRSTRLEN 46

uint32_t htonl(uint32_t);
uint16_t htons(uint16_t);
uint32_t ntohl(uint32_t);
uint16_t ntohs(uint16_t);

#define IPPROTO_IP       0
#define IPPROTO_HOPOPTS  0
#define IPPROTO_ICMP     1
#define IPPROTO_IGMP     2
#define IPPROTO_IPIP     4
#define IPPROTO_TCP      6
#define IPPROTO_EGP      8
#define IPPROTO_PUP      12
#define IPPROTO_UDP      17
#define IPPROTO_IDP      22
#define IPPROTO_TP       29
#define IPPROTO_DCCP     33
#define IPPROTO_IPV6     41
#define IPPROTO_ROUTING  43
#define IPPROTO_FRAGMENT 44
#define IPPROTO_RSVP     46
#define IPPROTO_GRE      47
#define IPPROTO_ESP      50
#define IPPROTO_AH       51
#define IPPROTO_ICMPV6   58
#define IPPROTO_NONE     59
#define IPPROTO_DSTOPTS  60
#define IPPROTO_MTP      92
#define IPPROTO_BEETPH   94
#define IPPROTO_ENCAP    98
#define IPPROTO_PIM      103
#define IPPROTO_COMP     108
#define IPPROTO_SCTP     132
#define IPPROTO_MH       135
#define IPPROTO_UDPLITE  136
#define IPPROTO_MPLS     137
#define IPPROTO_ETHERNET 143
#define IPPROTO_RAW      255
#define IPPROTO_MPTCP    262
#define IPPROTO_MAX      263

#define IPV6_ADDRFORM             1
#define IPV6_2292PKTINFO          2
#define IPV6_2292HOPOPTS          3
#define IPV6_2292DSTOPTS          4
#define IPV6_2292RTHDR            5
#define IPV6_2292PKTOPTIONS       6
#define IPV6_CHECKSUM             7
#define IPV6_2292HOPLIMIT         8
#define IPV6_NEXTHOP              9
#define IPV6_AUTHHDR              10
#define IPV6_UNICAST_HOPS         16
#define IPV6_MULTICAST_IF         17
#define IPV6_MULTICAST_HOPS       18
#define IPV6_MULTICAST_LOOP       19
#define IPV6_JOIN_GROUP           20
#define IPV6_LEAVE_GROUP          21
#define IPV6_ROUTER_ALERT         22
#define IPV6_MTU_DISCOVER         23
#define IPV6_MTU                  24
#define IPV6_RECVERR              25
#define IPV6_V6ONLY               26
#define IPV6_JOIN_ANYCAST         27
#define IPV6_LEAVE_ANYCAST        28
#define IPV6_MULTICAST_ALL        29
#define IPV6_ROUTER_ALERT_ISOLATE 30
#define IPV6_IPSEC_POLICY         34
#define IPV6_XFRM_POLICY          35
#define IPV6_HDRINCL              36

#define IN6ADDR_ANY_INIT                                       \
    {                                                          \
        {                                                      \
            {                                                  \
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 \
            }                                                  \
        }                                                      \
    }
#define IN6ADDR_LOOPBACK_INIT                                  \
    {                                                          \
        {                                                      \
            {                                                  \
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1 \
            }                                                  \
        }                                                      \
    }

typedef uint16_t in_port_t;
typedef uint32_t in_addr_t;

struct in_addr {
    in_addr_t s_addr;
};

struct sockaddr_in {
    sa_family_t sin_family;
    in_port_t sin_port;
    struct in_addr sin_addr;
    uint8_t sin_zero[8];
};

struct in6_addr {
    union {
        uint8_t __s6_addr[16];
        uint16_t __s6_addr16[8];
        uint32_t __s6_addr32[4];
    } __in6_union;
};
#define s6_addr   __in6_union.__s6_addr
#define s6_addr16 __in6_union.__s6_addr16
#define s6_addr32 __in6_union.__s6_addr32

struct sockaddr_in6 {
    sa_family_t sin6_family;
    in_port_t sin6_port;
    uint32_t sin6_flowinfo;
    struct in6_addr sin6_addr;
    uint32_t sin6_scope_id;
};

#endif // _NETINET_IN_H
