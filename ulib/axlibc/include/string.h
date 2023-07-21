#ifndef __STRING_H__
#define __STRING_H__

#include <stddef.h>

int atoi(const char *s);

void *memset(void *dest, int c, size_t n);
void *memchr(const void *src, int c, size_t n);

size_t strlen(const char *s);
size_t strnlen(const char *s, size_t n);

char *strcpy(char *restrict d, const char *restrict s);
char *strncpy(char *restrict d, const char *restrict s, size_t n);

char *strcat(char *restrict d, const char *restrict s);
char *strncat(char *restrict d, const char *restrict s, size_t n);

int strcmp(const char *l, const char *r);
int strncmp(const char *l, const char *r, size_t n);

int strcoll(const char *, const char *);

size_t strcspn(const char *s1, const char *s2);
size_t strspn(const char *s, const char *c);
char *strpbrk(const char *, const char *);

char *strchrnul(const char *, int);

char *strrchr(const char *str, int c);
char *strchr(const char *str, int c);

int strcasecmp(const char *__s1, const char *__s2);
int strncasecmp(const char *__s1, const char *__s2, size_t __n);

char *strstr(const char *h, const char *n);

char *strerror(int e);
int strerror_r(int, char *, size_t);

void *memcpy(void *restrict dest, const void *restrict src, size_t n);

void *memmove(void *dest, const void *src, size_t n);

int memcmp(const void *vl, const void *vr, size_t n);

char *strdup(const char *__s);

#endif // __STRING_H__
