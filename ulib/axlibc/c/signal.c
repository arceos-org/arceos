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
