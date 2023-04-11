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

#define unimplemented(fmt, ...)                                                                \
    printf("\x1b[33m%s%s:\x1b[0m " fmt "\n", "WARN: no ax_call implementation for ", __func__, \
           ##__VA_ARGS__)

struct _IO_FILE {
    int fd;
};

typedef struct _IO_FILE FILE;

#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

#define FILENAME_MAX 4096

#endif // __STDIO_H__
