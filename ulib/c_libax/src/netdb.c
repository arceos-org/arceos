#ifdef AX_CONFIG_NET

#include <netdb.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <libax.h>

static const char msgs[] = "Invalid flags\0"
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

const char *__lctrans_cur(const char *msg)
{
    return msg;
}

#define LCTRANS_CUR(msg) __lctrans_cur(msg)

const char *gai_strerror(int ecode)
{
    const char *s;
    for (s = msgs, ecode++; ecode && *s; ecode++, s++)
        for (; *s; s++)
            ;
    if (!*s)
        s++;
    return LCTRANS_CUR(s);
}

#endif // AX_CONFIG_NET
