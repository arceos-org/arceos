#ifndef _PTHREAD_H
#define _PTHREAD_H

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

#include <ax_pthread_mutex.h>

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

typedef void *pthread_t;

#define PTHREAD_CANCELED ((void *)-1)
#define SIGCANCEL        33

#if defined(AX_CONFIG_MULTITASK)

_Noreturn void pthread_exit(void *);
pthread_t pthread_self(void);
int pthread_create(pthread_t *__restrict, const pthread_attr_t *__restrict, void *(*)(void *),
                   void *__restrict);
int pthread_join(pthread_t t, void **res);

int pthread_setcancelstate(int, int *);
int pthread_setcanceltype(int, int *);

int pthread_mutex_init(pthread_mutex_t *__restrict, const pthread_mutexattr_t *__restrict);
int pthread_mutex_lock(pthread_mutex_t *);
int pthread_mutex_unlock(pthread_mutex_t *);

#endif // AX_CONFIG_MULTITASK

#endif // _PTHREAD_H
