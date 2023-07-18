#ifndef _DIRENT_H
#define _DIRENT_H

#include <sys/types.h>

struct __dirstream {
    long long tell;
    int fd;
    int buf_pos;
    int buf_end;
    int lock[1];
    char buf[2048];
};

typedef struct __dirstream DIR;

struct dirent {
    ino_t d_ino;
    off_t d_off;
    unsigned short d_reclen;
    unsigned char d_type;
    char d_name[256];
};

int closedir(DIR *);
DIR *fdopendir(int);
DIR *opendir(const char *);
struct dirent *readdir(DIR *);
int readdir_r(DIR *__restrict, struct dirent *__restrict, struct dirent **__restrict);
void rewinddir(DIR *);
int dirfd(DIR *);

#define DT_UNKNOWN 0
#define DT_FIFO    1
#define DT_CHR     2
#define DT_DIR     4
#define DT_BLK     6
#define DT_REG     8
#define DT_LNK     10
#define DT_SOCK    12
#define DT_WHT     14
#define IFTODT(x)  ((x) >> 12 & 017)
#define DTTOIF(x)  ((x) << 12)

#endif //_DIRENT_H
