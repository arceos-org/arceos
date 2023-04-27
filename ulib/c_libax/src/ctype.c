#include <ctype.h>
#include <stdio.h>
#include <string.h>

int isupper(int c)
{
    return (unsigned)c - 'A' < 26;
}

int tolower(int c)
{
    if (isupper(c))
        return c | 32;
    return c;
}

int islower(int c)
{
    return (unsigned)c - 'a' < 26;
}

int toupper(int c)
{
    if (islower(c))
        return c & 0x5f;
    return c;
}

int isgraph(int c)
{
    return (unsigned)c - 0x21 < 0x5e;
}

int isalpha(int c)
{
    return ((unsigned)c | 32) - 'a' < 26;
}

int isprint(int c)
{
    return (unsigned)c - 0x20 < 0x5f;
}

int isalnum(int c)
{
    return isalpha(c) || isdigit(c);
}

int iscntrl(int c)
{
    return (unsigned)c < 0x20 || c == 0x7f;
}

int ispunct(int c)
{
    return isgraph(c) && !isalnum(c);
}

int isxdigit(int c)
{
    return isdigit(c) || ((unsigned)c | 32) - 'a' < 6;
}

int isascii(int c)
{
    return !(c & ~0x7f);
}