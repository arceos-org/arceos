#ifndef _SIGNAL_H
#define _SIGNAL_H

#include <pthread.h>
#include <stddef.h>
#include <stdint.h>

typedef int sig_atomic_t;

union sigval {
    int sival_int;
    void *sival_ptr;
};

typedef union sigval __sigval_t;

#define SA_NOCLDSTOP 1
#define SA_NOCLDWAIT 2
#define SA_SIGINFO   4
#define SA_ONSTACK   0x08000000
#define SA_RESTART   0x10000000
#define SA_NODEFER   0x40000000
#define SA_RESETHAND 0x80000000
#define SA_RESTORER  0x04000000

#define SIG_BLOCK   0
#define SIG_UNBLOCK 1
#define SIG_SETMASK 2

#define SI_ASYNCNL (-60)
#define SI_TKILL   (-6)
#define SI_SIGIO   (-5)
#define SI_ASYNCIO (-4)
#define SI_MESGQ   (-3)
#define SI_TIMER   (-2)
#define SI_QUEUE   (-1)
#define SI_USER    0
#define SI_KERNEL  128

typedef struct {
    int si_signo, si_errno, si_code;
    union {
        char __pad[128 - 2 * sizeof(int) - sizeof(long)];
        struct {
            union {
                struct {
                    int si_pid;
                    unsigned int si_uid;
                } __piduid;
                struct {
                    int si_timerid;
                    int si_overrun;
                } __timer;
            } __first;
            union {
                union sigval si_value;
                struct {
                    int si_status;
                    long si_utime, si_stime;
                } __sigchld;
            } __second;
        } __si_common;
        struct {
            void *si_addr;
            short si_addr_lsb;
            union {
                struct {
                    void *si_lower;
                    void *si_upper;
                } __addr_bnd;
                unsigned si_pkey;
            } __first;
        } __sigfault;
        struct {
            long si_band;
            int si_fd;
        } __sigpoll;
        struct {
            void *si_call_addr;
            int si_syscall;
            unsigned si_arch;
        } __sigsys;
    } __si_fields;
} siginfo_t;

#define si_pid       __si_fields.__si_common.__first.__piduid.si_pid
#define si_uid       __si_fields.__si_common.__first.__piduid.si_uid
#define si_status    __si_fields.__si_common.__second.__sigchld.si_status
#define si_utime     __si_fields.__si_common.__second.__sigchld.si_utime
#define si_stime     __si_fields.__si_common.__second.__sigchld.si_stime
#define si_value     __si_fields.__si_common.__second.si_value
#define si_addr      __si_fields.__sigfault.si_addr
#define si_addr_lsb  __si_fields.__sigfault.si_addr_lsb
#define si_lower     __si_fields.__sigfault.__first.__addr_bnd.si_lower
#define si_upper     __si_fields.__sigfault.__first.__addr_bnd.si_upper
#define si_pkey      __si_fields.__sigfault.__first.si_pkey
#define si_band      __si_fields.__sigpoll.si_band
#define si_fd        __si_fields.__sigpoll.si_fd
#define si_timerid   __si_fields.__si_common.__first.__timer.si_timerid
#define si_overrun   __si_fields.__si_common.__first.__timer.si_overrun
#define si_ptr       si_value.sival_ptr
#define si_int       si_value.sival_int
#define si_call_addr __si_fields.__sigsys.si_call_addr
#define si_syscall   __si_fields.__sigsys.si_syscall
#define si_arch      __si_fields.__sigsys.si_arch

#define SIGHUP    1
#define SIGINT    2
#define SIGQUIT   3
#define SIGILL    4
#define SIGTRAP   5
#define SIGABRT   6
#define SIGIOT    SIGABRT
#define SIGBUS    7
#define SIGFPE    8
#define SIGKILL   9
#define SIGUSR1   10
#define SIGSEGV   11
#define SIGUSR2   12
#define SIGPIPE   13
#define SIGALRM   14
#define SIGTERM   15
#define SIGSTKFLT 16
#define SIGCHLD   17
#define SIGCONT   18
#define SIGSTOP   19
#define SIGTSTP   20
#define SIGTTIN   21
#define SIGTTOU   22
#define SIGURG    23
#define SIGXCPU   24
#define SIGXFSZ   25
#define SIGVTALRM 26
#define SIGPROF   27
#define SIGWINCH  28
#define SIGIO     29
#define SIGPOLL   29
#define SIGPWR    30
#define SIGSYS    31
#define SIGUNUSED SIGSYS

#define _NSIG 65

typedef void (*sighandler_t)(int);
#define SIG_ERR ((void (*)(int)) - 1)
#define SIG_DFL ((void (*)(int))0)
#define SIG_IGN ((void (*)(int))1)

typedef struct __sigset_t {
    unsigned long __bits[128 / sizeof(long)];
} sigset_t;

struct sigaction {
    union {
        void (*sa_handler)(int);
        void (*sa_sigaction)(int, siginfo_t *, void *);
    } __sa_handler;
    sigset_t sa_mask;
    int sa_flags;
    void (*sa_restorer)(void);
};

#define sa_handler   __sa_handler.sa_handler
#define sa_sigaction __sa_handler.sa_sigaction

void (*signal(int, void (*)(int)))(int);
int sigaction(int, const struct sigaction *__restrict, struct sigaction *__restrict);
int sigemptyset(sigset_t *);
int raise(int);
int sigaddset(sigset_t *, int);
int pthread_sigmask(int, const sigset_t *__restrict, sigset_t *__restrict);

int kill(pid_t, int);

#ifdef AX_CONFIG_MULTITASK
int pthread_kill(pthread_t t, int sig);
#endif

#endif // _SIGNAL_H
