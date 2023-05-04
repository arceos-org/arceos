#include <ctype.h>
#include <stdio.h>
#include <string.h>

int tolower(int c)
{
    if (isupper(c))
        return c | 32;
    return c;
}

int toupper(int c)
{
    if (islower(c))
        return c & 0x5f;
    return c;
}
