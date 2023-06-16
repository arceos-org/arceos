#ifndef __STDIO_H__
#define __STDIO_H__

#include <stdarg.h>
#include <stddef.h>

#define FILE_BUF_SIZE 1024
// TODO: complete this struct
struct IO_FILE {
    int fd;
    uint16_t buffer_len;
    char buf[FILE_BUF_SIZE];
};

typedef struct IO_FILE FILE;

extern FILE *const stdin;
extern FILE *const stdout;
extern FILE *const stderr;

#define stdin  (stdin)
#define stdout (stdout)
#define stderr (stderr)

#define EOF (-1)

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

#if defined(AX_CONFIG_ALLOC) && defined(AX_CONFIG_FS)
FILE *fopen(const char *filename, const char *mode);
char *fgets(char *__restrict, int, FILE *__restrict);
#endif

int fflush(FILE *);

int getchar();

int fputc(int, FILE *);
int putc(int, FILE *);
int putchar(int);

int fputs(const char *__restrict, FILE *__restrict);
int puts(const char *s);

int printf(const char *__restrict, ...);
int fprintf(FILE *__restrict, const char *__restrict, ...);
int sprintf(char *__restrict, const char *__restrict, ...);
int snprintf(char *__restrict, size_t, const char *__restrict, ...);

int vfprintf(FILE *__restrict, const char *__restrict, va_list);
int vsprintf(char *__restrict, const char *__restrict, va_list);
int vsnprintf(char *__restrict, size_t, const char *__restrict, va_list);

void perror(const char *);

#endif // __STDIO_H__
