#include <libax.h>
#include <pthread.h>
#include <unistd.h>

#if defined(AX_CONFIG_MULTITASK)
_Noreturn void pthread_exit(void *result)
{
    unimplemented();
}

pthread_t pthread_self(void)
{
    return (pthread_t)getpid();
}

int pthread_create(pthread_t *restrict res, const pthread_attr_t *restrict attrp,
                   void *(*entry)(void *), void *restrict arg)
{
    unimplemented();
    return 0;
}

int pthread_join(pthread_t t, void **res)
{
    unimplemented();
    return 0;
}
#endif
