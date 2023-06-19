#include <string.h>
#include <unistd.h>

char **environ = 0;

char *getenv(const char *name)
{
    size_t l = strchrnul(name, '=') - name;
    if (l && !name[l] && environ)
        for (char **e = environ; *e; e++)
            if (!strncmp(name, *e, l) && l[*e] == '=')
                return *e + l + 1;
    return 0;
}
