#include <libax.h>
#include <pthread.h>
#include <unistd.h>

#if defined(AX_CONFIG_MULTITASK)

_Noreturn void pthread_exit(void *result)
{
    ax_pthread_exit(result);
}

pthread_t pthread_self(void)
{
    return ax_pthread_self();
}

int pthread_create(pthread_t *restrict res, const pthread_attr_t *restrict attrp,
                   void *(*entry)(void *), void *restrict arg)
{
    return ax_pthread_create(res, attrp, (void *)entry, arg);
}

int pthread_join(pthread_t t, void **res)
{
    return ax_pthread_join(t, res);
}

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

int pthread_mutex_init(pthread_mutex_t *restrict m, const pthread_mutexattr_t *restrict a)
{
    ax_pthread_mutex_init(m, a);
    return 0;
}

int pthread_mutex_lock(pthread_mutex_t *m)
{
    ax_pthread_mutex_lock(m);
    return 0;
}

int pthread_mutex_unlock(pthread_mutex_t *m)
{
    ax_pthread_mutex_unlock(m);
    return 0;
}

#endif // AX_CONFIG_MULTITASK
