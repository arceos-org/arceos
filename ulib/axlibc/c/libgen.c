#include <libgen.h>
#include <string.h>

char *dirname(char *s)
{
    size_t i;
    if (!s || !*s)
        return ".";
    i = strlen(s) - 1;
    for (; s[i] == '/'; i--)
        if (!i)
            return "/";
    for (; s[i] != '/'; i--)
        if (!i)
            return ".";
    for (; s[i] == '/'; i--)
        if (!i)
            return "/";
    s[i + 1] = 0;
    return s;
}

char *basename(char *s)
{
    size_t i;
    if (!s || !*s)
        return ".";
    i = strlen(s) - 1;
    for (; i && s[i] == '/'; i--) s[i] = 0;
    for (; i && s[i - 1] != '/'; i--)
        ;
    return s + i;
}
