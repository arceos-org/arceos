#ifndef __STDIO_H__
#define __STDIO_H__

#include <stdarg.h>
#include <stddef.h>

// TODO: complete this struct
struct IO_FILE {
    int fd;
};

typedef struct IO_FILE FILE;

extern FILE *const stdin;
extern FILE *const stdout;
extern FILE *const stderr;

#define stdin  (stdin)
#define stdout (stdout)
#define stderr (stderr)

#define EOF (-1)

#define printf(...) fprintf(stdout->fd, __VA_ARGS__)

#define unimplemented(fmt, ...)                                                                \
    printf("\x1b[33m%s%s:\x1b[0m " fmt "\n", "WARN: no ax_call implementation for ", __func__, \
           ##__VA_ARGS__)

#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

#define F_EOF  16
#define F_ERR  32
#define F_SVB  64
#define F_NORD 4
#define UNGET  8

#define FILENAME_MAX 4096

int getchar();
int putchar(int);
int puts(const char *s);
void fprintf(int f, const char *fmt, ...);

#if defined(AX_CONFIG_ALLOC) && defined(AX_CONFIG_FS)
FILE *fopen(const char *filename, const char *mode);
#endif

int fflush(FILE *);

int vsnprintf(char *__restrict, size_t, const char *__restrict, va_list);
int snprintf(char *__restrict, size_t, const char *__restrict, ...);
int vsprintf(char *__restrict, const char *__restrict, va_list);
int vfprintf(FILE *__restrict, const char *__restrict, va_list);
int sprintf(char *__restrict, const char *__restrict, ...);

#endif // __STDIO_H__
