#ifndef __STDLIB_H__
#define __STDLIB_H__

#include <stddef.h>

int rand(void);
void srand(unsigned);

#ifdef AX_CONFIG_ALLOC
void *malloc(size_t size);
void free(void *addr);
void *calloc(size_t nmemb, size_t size);
void *realloc(void *memblock, size_t size);
#endif

_Noreturn void abort(void);
char *getenv(const char *name);

#ifdef AX_CONFIG_FP_SIMD
float strtof(const char *__restrict, char **__restrict);
double strtod(const char *__restrict, char **__restrict);
#endif

long strtol(const char *__restrict, char **__restrict, int);
unsigned long strtoul(const char *nptr, char **endptr, int base);
long long strtoll(const char *nptr, char **endptr, int base);
unsigned long long strtoull(const char *nptr, char **endptr, int base);

long long atoll(const char *nptr);

void exit(int);

long long llabs(long long x);
int abs(int x);

#endif //__STDLIB_H__
