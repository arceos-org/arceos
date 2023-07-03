#ifndef _SYS_WAIT_H
#define _SYS_WAIT_H

#include <sys/resource.h>
#include <sys/types.h>

#define WNOHANG 1

pid_t waitpid(pid_t __pid, int *__stat_loc, int __options);
pid_t wait3(int *, int, struct rusage *);

#endif
