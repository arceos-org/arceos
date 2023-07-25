#ifndef __STDLIB_H__
#define __STDLIB_H__

#include <stddef.h>

long long atoll(const char *nptr);

float strtof(const char *__restrict, char **__restrict);
double strtod(const char *__restrict, char **__restrict);

long strtol(const char *__restrict, char **__restrict, int);
unsigned long strtoul(const char *nptr, char **endptr, int base);
long long strtoll(const char *nptr, char **endptr, int base);
unsigned long long strtoull(const char *nptr, char **endptr, int base);

int rand(void);
void srand(unsigned);

void *malloc(size_t);
void *calloc(size_t, size_t);
void *realloc(void *, size_t);
void free(void *);

_Noreturn void abort(void);
_Noreturn void exit(int);

char *getenv(const char *);

int abs(int);
long labs(long);
long long llabs(long long);

#endif //__STDLIB_H__
