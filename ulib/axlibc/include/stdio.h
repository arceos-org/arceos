#ifndef __STDIO_H__
#define __STDIO_H__

#include <stdarg.h>
#include <stddef.h>

#define _IOFBF 0
#define _IOLBF 1
#define _IONBF 2

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

#if defined(AX_LOG_WARN) || defined(AX_LOG_INFO) || defined(AX_LOG_DEBUG) || defined(AX_LOG_TRACE)

#define unimplemented(fmt, ...)                                                                \
    printf("\x1b[33m%s%s:\x1b[0m " fmt "\n", "WARN: no ax_call implementation for ", __func__, \
           ##__VA_ARGS__)
#else

#define unimplemented(fmt, ...) \
    do {                        \
    } while (0)

#endif

#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

#define F_EOF  16
#define F_ERR  32
#define F_SVB  64
#define F_NORD 4
#define UNGET  8

#define FILENAME_MAX 4096
#define BUFSIZ       1024
#define L_tmpnam     20

FILE *fopen(const char *filename, const char *mode);
FILE *freopen(const char *__restrict, const char *__restrict, FILE *__restrict);
int fclose(FILE *);

int remove(const char *);
int rename(const char *, const char *);

int feof(FILE *__stream);
int ferror(FILE *);
int fflush(FILE *);
void clearerr(FILE *);

int fseek(FILE *__stream, long __off, int __whence);
long ftell(FILE *);

size_t fread(void *__restrict, size_t, size_t, FILE *__restrict);
size_t fwrite(const void *__restrict, size_t, size_t, FILE *__restrict);

int getc(FILE *);
int getchar(void);
int ungetc(int, FILE *);

int fputc(int, FILE *);
int putc(int, FILE *);
int putchar(int);

char *fgets(char *__restrict, int, FILE *__restrict);

int fputs(const char *__restrict, FILE *__restrict);
int puts(const char *s);

int printf(const char *__restrict, ...);
int fprintf(FILE *__restrict, const char *__restrict, ...);
int sprintf(char *__restrict, const char *__restrict, ...);
int snprintf(char *__restrict, size_t, const char *__restrict, ...);

int vfprintf(FILE *__restrict, const char *__restrict, va_list);
int vsprintf(char *__restrict, const char *__restrict, va_list);
int vsnprintf(char *__restrict, size_t, const char *__restrict, va_list);

int fscanf(FILE *__restrict, const char *__restrict, ...);
int sscanf(const char *__restrict, const char *__restrict, ...);

void perror(const char *);

int setvbuf(FILE *__restrict, char *__restrict, int, size_t);

char *tmpnam(char *);
FILE *tmpfile(void);

FILE *fdopen(int, const char *);
int fileno(FILE *);
off_t ftello(FILE *);

int getc_unlocked(FILE *);
ssize_t getdelim(char **__restrict, size_t *__restrict, int, FILE *__restrict);
ssize_t getline(char **__restrict, size_t *__restrict, FILE *__restrict);

#endif // __STDIO_H__
