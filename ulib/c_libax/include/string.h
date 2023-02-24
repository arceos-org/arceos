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
int strcmp(const char *l, const char *r);
char *strncpy(char *restrict d, const char *restrict s, size_t n);
int strncmp(const char *_l, const char *_r, size_t n);

#endif // __STRING_H__
