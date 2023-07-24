#include <errno.h>
#include <signal.h>
#include <stddef.h>
#include <stdio.h>

int sigaction_helper(int signum, const struct sigaction *act, struct sigaction *oldact,
                     size_t sigsetsize)
{
    if (signum == SIGKILL || signum == SIGSTOP)
        return -EINVAL;

    if (oldact)
        *oldact = (struct sigaction){0};

    return 0;
}

void (*signal(int signum, void (*handler)(int)))(int)
{
    struct sigaction old;
    struct sigaction act = {
        .sa_handler = handler, .sa_flags = SA_RESTART, /* BSD signal semantics */
    };

    if (sigaction_helper(signum, &act, &old, sizeof(sigset_t)) < 0)
        return SIG_ERR;

    return (old.sa_flags & SA_SIGINFO) ? NULL : old.sa_handler;
}

int sigaction(int sig, const struct sigaction *restrict act, struct sigaction *restrict oact)
{
    return sigaction_helper(sig, act, oact, sizeof(sigset_t));
}

// TODO
int kill(pid_t __pid, int __sig)
{
    unimplemented();
    return 0;
}

int sigemptyset(sigset_t *set)
{
    set->__bits[0] = 0;
    if (sizeof(long) == 4 || _NSIG > 65)
        set->__bits[1] = 0;
    if (sizeof(long) == 4 && _NSIG > 65) {
        set->__bits[2] = 0;
        set->__bits[3] = 0;
    }
    return 0;
}

// TODO
int raise(int __sig)
{
    unimplemented();
    return 0;
}

int sigaddset(sigset_t *set, int sig)
{
    unsigned s = sig - 1;
    if (s >= _NSIG - 1 || sig - 32U < 3) {
        errno = EINVAL;
        return -1;
    }
    set->__bits[s / 8 / sizeof *set->__bits] |= 1UL << (s & (8 * sizeof *set->__bits - 1));
    return 0;
}

// TODO
int pthread_sigmask(int __how, const sigset_t *restrict __newmask, sigset_t *restrict __oldmask)
{
    unimplemented();
    return 0;
}

#ifdef AX_CONFIG_MULTITASK
// TODO
int pthread_kill(pthread_t t, int sig)
{
    unimplemented();
    return 0;
}
#endif
