#include <axlibc.h>
#include <errno.h>
#include <stdio.h>
#include <sys/resource.h>

int getrlimit(int resource, struct rlimit *rlimits)
{
    switch (resource) {
    case RLIMIT_DATA:
        break;
    case RLIMIT_STACK:
        break;
    case RLIMIT_NOFILE:
        break;
    default:
        // Unsupported resource
        return -EINVAL;
    }

    if (!rlimits)
        return 0;
    switch (resource) {
    case RLIMIT_STACK:
        rlimits->rlim_cur = AX_CONFIG_TASK_STACK_SIZE;
        rlimits->rlim_max = AX_CONFIG_TASK_STACK_SIZE;
        break;
    case RLIMIT_NOFILE:
        rlimits->rlim_cur = AX_FILE_LIMIT;
        rlimits->rlim_max = AX_FILE_LIMIT;
        break;
    default:
        break;
    }
    return 0;
}

int setrlimit(int resource, struct rlimit *rlimits)
{
    switch (resource) {
    case RLIMIT_DATA:
        break;
    case RLIMIT_STACK:
        break;
    case RLIMIT_NOFILE:
        break;
    default:
        // Unsupported resource
        return -EINVAL;
    }
    // Set resouce
    if (rlimits) {
        switch (resource) {
        default:
            // Currently do not support set resources
            break;
        }
    }
    return 0;
}

// TODO
int getrusage(int __who, struct rusage *__usage)
{
    unimplemented();
    return 0;
}
