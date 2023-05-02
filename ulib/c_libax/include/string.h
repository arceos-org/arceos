#ifndef __STRING_H__
#define __STRING_H__

#include <stdint.h>

int isspace(int c);
int isdigit(int c);
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

size_t strcspn(const char *s1, const char *s2);

char *strrchr(const char *str, int c);
char *strchr(const char *str, int c);

char *strerror(int n);

#endif // __STRING_H__
