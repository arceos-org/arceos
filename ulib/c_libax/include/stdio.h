#ifndef __STDIO_H__
#define __STDIO_H__

#define stdin  0
#define stdout 1
#define stderr 2

int getchar();
int putchar(int);
int puts(const char *s);
void fprintf(int f, const char *fmt, ...);
int fflush(int);

#define EOF (-1)

#define printf(...) fprintf(stdout, __VA_ARGS__)

#endif // __STDIO_H__
