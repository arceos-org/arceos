#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

char *environ_[2] = {"dummy", NULL};
char **environ = (char **)environ_;

char *getenv(const char *name)
{
    size_t l = strchrnul(name, '=') - name;
    if (l && !name[l] && environ)
        for (char **e = environ; *e; e++)
            if (!strncmp(name, *e, l) && l[*e] == '=')
                return *e + l + 1;
    return 0;
}

// TODO
int setenv(const char *__name, const char *__value, int __replace)
{
    unimplemented();
    return 0;
}

// TODO
int unsetenv(const char *__name)
{
    unimplemented();
    return 0;
}
