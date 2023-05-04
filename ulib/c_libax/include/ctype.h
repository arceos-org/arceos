#ifndef _CTYPE_H
#define _CTYPE_H

int tolower(int __c);
int toupper(int __c);

#define isalpha(a)  ((((unsigned)(a) | 32) - 'a') < 26)
#define isdigit(a)  (((unsigned)(a) - '0') < 10)
#define islower(a)  (((unsigned)(a) - 'a') < 26)
#define isupper(a)  (((unsigned)(a) - 'A') < 26)
#define isprint(a)  (((unsigned)(a)-0x20) < 0x5f)
#define isgraph(a)  (((unsigned)(a)-0x21) < 0x5e)
#define isalnum(a)  ((isalpha(a) || isdigit(a)))
#define iscntrl(a)  (((unsigned)a < 0x20 || a == 0x7f))
#define ispunct(a)  ((isgraph(a) && !isalnum(a)))
#define isxdigit(a) ((isdigit(a) || ((unsigned)a | 32) - 'a' < 6))
#define isascii(a)  ((!(a & ~0x7f)))
#define isspace(a)  ((a == ' ' || (unsigned)a - '\t' < 5))

#endif
