#include <dirent.h>
#include <errno.h>
#include <fnmatch.h>
#include <glob.h>
#include <limits.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

struct match {
    struct match *next;
    char name[];
};

#if defined(AX_CONFIG_FS) && defined(AX_CONFIG_ALLOC)
//TODO
int glob(const char *restrict pat, int flags, int (*errfunc)(const char *path, int err),
         glob_t *restrict g)
{
    unimplemented();
    return 0;
}
#endif

#ifdef AX_CONFIG_ALLOC
void globfree(glob_t *g)
{
    unimplemented();
    return ;
}
#endif
