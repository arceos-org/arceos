#ifndef _CTYPE_H
#define _CTYPE_H

int tolower(int __c);
int toupper(int __c);

int isprint(int);
int isalpha(int c);
int isalnum(int);
int isupper(int c);
int islower(int c);

int isxdigit(int);
int isgraph(int);
int iscntrl(int);
int ispunct(int);

int isascii(int);

#endif
