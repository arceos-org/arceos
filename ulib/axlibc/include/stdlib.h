#ifndef __STDLIB_H__
#define __STDLIB_H__

#include <features.h>
#include <stddef.h>

#define RAND_MAX (0x7fffffff)

#define WEXITSTATUS(s) (((s)&0xff00) >> 8)
#define WTERMSIG(s)    ((s)&0x7f)
#define WIFEXITED(s)   (!WTERMSIG(s))
#define WIFSIGNALED(s) (((s)&0xffff) - 1U < 0xffu)

#define EXIT_FAILURE 1
#define EXIT_SUCCESS 0

long long atoll(const char *nptr);

float strtof(const char *__restrict, char **__restrict);
double strtod(const char *__restrict, char **__restrict);

long strtol(const char *__restrict, char **__restrict, int);
unsigned long strtoul(const char *nptr, char **endptr, int base);
long long strtoll(const char *nptr, char **endptr, int base);
unsigned long long strtoull(const char *nptr, char **endptr, int base);

int rand(void);
void srand(unsigned);
long random(void);
void srandom(unsigned int);

#ifdef AX_CONFIG_FP_SIMD
float strtof(const char *__restrict, char **__restrict);
double strtod(const char *__restrict, char **__restrict);
long double strtold(const char *__restrict, char **__restrict);
#endif

void qsort(void *, size_t, size_t, int (*)(const void *, const void *));

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

int mkstemp(char *);
int mkostemp(char *, int);
int setenv(const char *, const char *, int);
int unsetenv(const char *);
int system(const char *);

#endif //__STDLIB_H__
