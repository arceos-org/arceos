#ifdef AX_CONFIG_FS

#include <glob.h>
#include <stdio.h>

// TODO
int glob(const char *restrict pat, int flags, int (*errfunc)(const char *path, int err),
         glob_t *restrict g)
{
    unimplemented();
    return 0;
}

void globfree(glob_t *g)
{
    unimplemented();
    return;
}

#endif // AX_CONFIG_FS
