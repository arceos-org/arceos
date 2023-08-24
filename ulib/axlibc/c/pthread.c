#ifdef AX_CONFIG_MULTITASK

#include <errno.h>
#include <limits.h>
#include <pthread.h>
#include <stdio.h>
#include <unistd.h>

int pthread_setcancelstate(int new, int *old)
{
    unimplemented();
    return 0;
}

int pthread_setcanceltype(int new, int *old)
{
    unimplemented();
    return 0;
}

// TODO
void pthread_testcancel(void)
{
    unimplemented();
    return;
}

// TODO
int pthread_cancel(pthread_t t)
{
    unimplemented();
    return 0;
}

// TODO
int pthread_mutex_trylock(pthread_mutex_t *m)
{
    unimplemented();
    return 0;
}

// TODO
int pthread_setname_np(pthread_t thread, const char *name)
{
    unimplemented();
    return 0;
}

int pthread_cond_init(pthread_cond_t *restrict c, const pthread_condattr_t *restrict a)
{
    *c = (pthread_cond_t){0};
    if (a) {
        c->_c_clock = a->__attr & 0x7fffffff;
        if (a->__attr >> 31)
            c->_c_shared = (void *)-1;
    }
    return 0;
}

// TODO
int pthread_cond_signal(pthread_cond_t *__cond)
{
    unimplemented();
    return 0;
}

// TODO
int pthread_cond_wait(pthread_cond_t *__restrict__ __cond, pthread_mutex_t *__restrict__ __mutex)
{
    unimplemented();
    return 0;
}

// TODO
int pthread_cond_broadcast(pthread_cond_t *c)
{
    unimplemented();
    return 0;
}

#define DEFAULT_STACK_SIZE 131072
#define DEFAULT_GUARD_SIZE 8192

// TODO
int pthread_attr_init(pthread_attr_t *a)
{
    *a = (pthread_attr_t){0};
    // __acquire_ptc();
    a->_a_stacksize = DEFAULT_STACK_SIZE;
    a->_a_guardsize = DEFAULT_GUARD_SIZE;
    // __release_ptc();
    return 0;
}

int pthread_attr_getstacksize(const pthread_attr_t *restrict a, size_t *restrict size)
{
    *size = a->_a_stacksize;
    return 0;
}

int pthread_attr_setstacksize(pthread_attr_t *a, size_t size)
{
    if (size - PTHREAD_STACK_MIN > SIZE_MAX / 4)
        return EINVAL;
    a->_a_stackaddr = 0;
    a->_a_stacksize = size;
    return 0;
}

#endif // AX_CONFIG_MULTITASK
