#ifndef __LWIPOPTS_H__
#define __LWIPOPTS_H__

#define NO_SYS 1
// #define LWIP_TIMERS 1
#define NO_SYS_NO_TIMERS 1

#define IP_DEFAULT_TTL       64
#define LWIP_ETHERNET        1
#define LWIP_ARP             1
#define ARP_QUEUEING         0
#define IP_FORWARD           0
#define LWIP_ICMP            1
#define LWIP_RAW             1
#define LWIP_DHCP            0
#define LWIP_AUTOIP          0
#define LWIP_SNMP            0
#define LWIP_IGMP            0
#define LWIP_DNS             0
#define LWIP_UDP             1
#define LWIP_UDPLITE         0
#define LWIP_TCP             1
#define LWIP_CALLBACK_API    1
#define LWIP_NETIF_API       0
#define LWIP_NETIF_LOOPBACK  0
#define LWIP_HAVE_LOOPIF     1
#define LWIP_HAVE_SLIPIF     0
#define LWIP_NETCONN         0
#define LWIP_SOCKET          0
#define PPP_SUPPORT          0
#define LWIP_IPV4            1
#define LWIP_IPV6            1
#define LWIP_IPV6_MLD        0
#define LWIP_IPV6_AUTOCONFIG 1

#define MEMP_NUM_TCP_PCB 1024

// disable checksum checks
#define CHECKSUM_CHECK_IP    0
#define CHECKSUM_CHECK_UDP   0
#define CHECKSUM_CHECK_TCP   0
#define CHECKSUM_CHECK_ICMP  0
#define CHECKSUM_CHECK_ICMP6 0

#define LWIP_CHECKSUM_ON_COPY 1

#define TCP_MSS     1460
#define TCP_WND     (32 * TCP_MSS)
#define TCP_SND_BUF (8 * TCP_MSS)

#define MEM_SIZE (2 * 1024 * 1024)

#define MEMP_NUM_TCP_SEG 256
#define PBUF_POOL_SIZE   512

// #define TCP_MSS 1460
// #define TCP_WND (16 * TCP_MSS)
// #define TCP_SND_BUF (8 * TCP_MSS)
// #define MEM_LIBC_MALLOC 1
// #define MEMP_MEM_MALLOC 1

#define SYS_LIGHTWEIGHT_PROT 0
#define LWIP_DONT_PROVIDE_BYTEORDER_FUNCTIONS

// needed on 64-bit systems, enable it always so that the same configuration
// is used regardless of the platform
#define IPV6_FRAG_COPYHEADER 1

#define LWIP_DEBUG        0

#define LWIP_DBG_TYPES_ON LWIP_DBG_OFF

#define INET_DEBUG        LWIP_DBG_ON
#define IP_DEBUG          LWIP_DBG_ON
#define RAW_DEBUG         LWIP_DBG_ON
#define SYS_DEBUG         LWIP_DBG_ON
#define NETIF_DEBUG       LWIP_DBG_ON
#define TCP_DEBUG         LWIP_DBG_ON
#define UDP_DEBUG         LWIP_DBG_ON
#define TCP_INPUT_DEBUG   LWIP_DBG_ON
#define TCP_OUTPUT_DEBUG  LWIP_DBG_ON
#define TCPIP_DEBUG       LWIP_DBG_ON
#define IP6_DEBUG         LWIP_DBG_ON
#define PBUF_DEBUG        LWIP_DBG_ON

#define LWIP_STATS         0
#define LWIP_STATS_DISPLAY 0
#define LWIP_PERF          0

#endif /* __LWIPOPTS_H__ */