#ifndef _PTHREAD_H
#define _PTHREAD_H 1

#include <locale.h>
#include <signal.h>
#include <stddef.h>
#include <stdint.h>

#define PTHREAD_CANCEL_ENABLE  0
#define PTHREAD_CANCEL_DISABLE 1
#define PTHREAD_CANCEL_MASKED  2

#define PTHREAD_CANCEL_DEFERRED     0
#define PTHREAD_CANCEL_ASYNCHRONOUS 1

typedef struct {
    unsigned __attr;
} pthread_condattr_t;

#define __DEFINED_pthread_condattr_t
#define __SIZEOF_PTHREAD_MUTEX_T 40

typedef struct {
    union {
        int __i[sizeof(long) == 8 ? 10 : 6];
        volatile int __vi[sizeof(long) == 8 ? 10 : 6];
        volatile void *volatile __p[sizeof(long) == 8 ? 5 : 6];
    } __u;
} pthread_mutex_t;

#define _m_type __u.__i[0]

typedef struct {
    unsigned __attr;
} pthread_mutexattr_t;

typedef struct {
    union {
        int __i[sizeof(long) == 8 ? 14 : 9];
        volatile int __vi[sizeof(long) == 8 ? 14 : 9];
        unsigned long __s[sizeof(long) == 8 ? 7 : 9];
    } __u;
} pthread_attr_t;
#define _a_stacksize __u.__s[0]
#define _a_guardsize __u.__s[1]
#define _a_stackaddr __u.__s[2]

typedef struct {
    union {
        int __i[12];
        volatile int __vi[12];
        void *__p[12 * sizeof(int) / sizeof(void *)];
    } __u;
} pthread_cond_t;
#define _c_clock  __u.__i[4]
#define _c_shared __u.__p[0]

#define PTHREAD_MUTEX_INITIALIZER \
    {                             \
        0                         \
    }

#define pthread __pthread

struct pthread {
    int tid;
    void *result;
    int errno_val;
};

typedef unsigned long pthread_t;

#define PTHREAD_CANCELED ((void *)-1)
#define SIGCANCEL        33

#if defined(AX_CONFIG_MULTITASK) && defined(AX_CONFIG_ALLOC)
_Noreturn void pthread_exit(void *);
pthread_t pthread_self(void);
int pthread_create(pthread_t *__restrict, const pthread_attr_t *__restrict, void *(*)(void *),
                   void *__restrict);
int pthread_join(pthread_t t, void **res);
#endif

#endif